#!/usr/bin/env python

import argparse
import csv
import logging
import typing

from dataclasses import dataclass


LOG = logging.getLogger(__name__)
formatter = logging.Formatter("%(asctime)s - %(levelname)s - %(message)s")
ch = logging.StreamHandler()
ch.setFormatter(formatter)
LOG.addHandler(ch)


@dataclass
class Half:
    """ One half of the transaction. """

    # Amount of the half. e.g. 169.84 AUD or 0.00060145 BTC.
    amount: float
    # Unit the Half is in, e.g. BTC or AUD.
    unit: str
    # Rate this half was bought or sold at.
    rate: float

    def subtract_sell(self, sell) -> (float, bool):
        """
        Return: (capital_gain, exhausted)
        Return: (capital gain from this sale, whether the half is exhausted)

        This function assumes that this Half (self) is a buy, and then takes
        in another Half assuming it is a sell, and figures out how much is
        left of this half (if any) and what the capital gain of this sell is.

        Doing this modifies the half. If the sell is less than the lot, the
        lot will have some amount left and `exhausted` will be False.
        If the sell is greater than the lot, `exhausted` will be True and
        the calling code should assume the Half is done and pop it off the stack.

        This function will also modify the sell Half based on how much if it could
        be reduced by consuming this buy Half.

        Returns the capital gain in terms of AUD.
        """
        delta = min(self.amount, sell.amount)
        LOG.debug(f"Subtracting {sell} from {self}, decreasing both by {delta}")
        remaining_buy = self.amount - sell.amount
        buy_in_aud = delta * self.rate
        sell_in_aud = delta * sell.rate
        if remaining_buy > 0:
            exhausted = False
            self.amount -= sell.amount
            sell.amount = 0
        else:
            exhausted = True
            sell.amount -= self.amount
            self.amount = 0
        LOG.debug(f"Buy is now {self} and sell is now {sell}")
        return (sell_in_aud - buy_in_aud), exhausted


@dataclass
class Transfer(Half):
    """ A Transfer in or out to / from this exchange. """

    # Amount. Should be positive.
    amount: float
    # Unit the Transfer is in, e.g. BTC or AUD.
    unit: str
    # True if this was a transfer out, False if it was a transfer in.
    outward: bool
    # Rate this transfer was made at.
    # NOTE: This is just an estimate made based on the last rate we saw for this currency.
    rate: float

    @classmethod
    def from_row(cls, row, rates):
        """ Takes a row and the dict of last rates we saw for each currency. """
        is_outward = bool(row["debit"])
        amount = row["debit"] if is_outward else row["credit"]
        amount = float(amount.replace(",", ""))
        unit = row["currency"]
        rate = rates[unit]
        return Transfer(amount=amount, unit=unit, outward=is_outward, rate=rate)


@dataclass
class Transaction:
    """ One full transaction. Assumes that one half of the transaction is in AUD. """

    # Whatever was sold in the transaction.
    sold: Half
    # What was aquired in the transaction.
    bought: Half

    @classmethod
    def from_rows(cls, first_row, second_row):
        rate = float(first_row["rates"].split("$")[1].split(" ")[0].replace(",", ""))
        first_is_debit = bool(first_row["debit"])
        if first_is_debit:
            first_amount = first_row["debit"]
            second_amount = second_row["credit"]
        else:
            first_amount = first_row["credit"]
            second_amount = second_row["debit"]
        first_amount = float(first_amount.replace(",", ""))
        second_amount = float(second_amount.replace(",", ""))
        first = Half(amount=first_amount, unit=first_row["currency"], rate=rate)
        second = Half(amount=second_amount, unit=second_row["currency"], rate=rate)
        if first_is_debit:
            sold = first
            bought = second
        else:
            sold = second
            bought = first
        return Transaction(sold=sold, bought=bought)

    def get_non_aud_unit(self):
        if self.sold.unit != "AUD":
            return self.sold.unit
        return self.bought.unit


def load_transactions(data_path):
    # Transactions are buys and sells from within the platform.
    # Transfer are when the user received (assumed to be buys) or sent
    # (assumed to be sells) some currency to some other exchange / wallet.
    transactions = []
    with open(data_path, "r", newline="") as f:
        reader = csv.DictReader(f)
        i = 0
        rows = [r for r in reader]
        # This dictionary contains the previous rate we saw for a particular currency.
        rates = {}
        while i < len(rows):
            first = rows[i]
            i += 1
            if "Received" in first["action"] or "Sent" in first["action"]:
                # This is a Transfer.
                t = Transfer.from_row(first, rates)
            elif not first["rates"]:
                # This is not a Transfer or a Transaction.
                continue
            else:
                # This is a Transaction, consume the next row.
                second = rows[i]
                i += 1
                t = Transaction.from_rows(first, second)
                rate = t.bought.rate
                unit = t.bought.unit
                rates[t.get_non_aud_unit()] = t.bought.rate
            transactions.append(t)
    return transactions


def get_currencies(transactions):
    """ Determine all currencies traded in, excluding AUD. """
    currencies = set()
    for t in transactions:
        try:
            currencies.add(t.sold.unit)
            currencies.add(t.bought.unit)
        except:
            pass
    currencies.remove("AUD")
    return currencies


def get_transactions_for_currency(transactions, currency):
    new_transactions = []
    for t in transactions:
        if isinstance(t, Transfer):
            if t.unit != currency:
                continue
        else:
            if not (t.sold.unit == currency or t.bought.unit == currency):
                continue
        new_transactions.append(t)
    transactions = new_transactions
    return transactions



def calculate_capital_gain(transactions, currency):
    """
    Warning: It is possible that a user will receive some amount of a currency
    from another exchange, then sell more in this exchange than they ever bought
    here. This will cause the capital gain calculation to fail. To resolve this,
    we would need to know at what price the client bought the currency that they
    exchanged in to this exchange.
    """
    LOG.debug(f"Determining capital gain for {currency}")
    lots = []  # This is used as a stack.
    capital_gain = 0
    
    for t in transactions:
        if isinstance(t, Transfer):
            if t.outward:
                bought = None
                sold = t
            else:
                bought = t
                sold = None
        else:
            if t.bought.unit == currency:
                bought = t.bought
                sold = None
            else:
                bought = None
                sold = t.sold
        if bought:
            lots.append(bought)
        else:
            while sold.amount > 0:
                try:
                    if sold.amount < 0.0000001:
                        # Catch the case where sold is tiny due to floating point rounding errors.
                        break
                    capital_gain_delta, exhausted = lots[-1].subtract_sell(sold)
                except IndexError:
                    raise RuntimeError(
                        f"Ran out of lots for {currency}. "
                        f"This means more was sold than was ever bought"
                    )
                capital_gain += capital_gain_delta
                if sold.amount < 0 or lots[-1].amount < 0:
                    raise RuntimeError(
                        "Somehow buy or sell half went below zero. "
                        "This is a logic error."
                    )
                if exhausted:
                    LOG.debug(f"Popping {lots[-1]}")
                    lots.pop()
                LOG.debug(f"Capital gain is {capital_gain}")
    return capital_gain


def parse_args():
    parser = argparse.ArgumentParser()
    parser.add_argument("data_path")
    parser.add_argument("-d", "--debug", action="store_true")
    parser.add_argument("--allowlist", nargs="*")
    args = parser.parse_args()
    args.allowlist = args.allowlist or []
    return args


def main():
    args = parse_args()

    if args.debug:
        LOG.setLevel("DEBUG")
    else:
        LOG.setLevel("INFO")

    transactions = load_transactions(args.data_path)

    LOG.debug(f"Loaded {len(transactions)} transactions")

    currencies = get_currencies(transactions)

    LOG.debug(f"Calculating capital gain for these currencies: {currencies}")

    for currency in currencies:
        if args.allowlist and currency not in args.allowlist:
            LOG.debug(f"Ignoring {currency} because it is not in the allowlist")
            continue
        ts = get_transactions_for_currency(transactions, currency)
        for t in ts:
            LOG.debug(t)
        capital_gain = calculate_capital_gain(ts, currency)
        LOG.info(f"Capital gain for {currency} is {capital_gain:.2f} AUD")


if __name__ == "__main__":
    main()
