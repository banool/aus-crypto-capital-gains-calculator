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
        remaining_buy = self.amount - sell.amount
        if remaining_buy > 0:
            exhausted = False
            self.amount -= sell.amount
            sell.amount = 0
        else:
            exhausted = True
            self.amount = 0
            sell.amount -= self.amount
        buy_in_aud = self.amount * self.rate
        sell_in_aud = sell.amount * sell.rate
        return (sell_in_aud - buy_in_aud), exhausted


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



def parse_args():
    parser = argparse.ArgumentParser()
    parser.add_argument("data_path")
    parser.add_argument("-d", "--debug", action="store_true")
    args = parser.parse_args()
    return args


def load_transactions(data_path):
    transactions = []
    with open(data_path, "r", newline='') as f:
        reader = csv.DictReader(f)
        i = 0
        rows = [r for r in reader]
        while i < len(rows):
            first = rows[i]
            i += 1
            if not first["rates"]:
                # Ignore any row that is not the first of the two rows that make
                # up a transaction.
                continue
            second = rows[i]
            t = Transaction.from_rows(first, second)
            transactions.append(t)
    return transactions


def get_currencies(transactions):
    """ Determine all currencies traded in, excluding AUD. """
    currencies = set()
    for t in transactions:
        currencies.add(t.sold.unit)
        currencies.add(t.bought.unit)
    currencies.remove("AUD")
    return currencies


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
        if not (t.sold.unit == currency or t.bought.unit == currency):
            # Ignore transactions not involving this currency.
            continue
        if t.bought.unit == currency:
            lots.append(t.bought)
        else:
            while t.sold.amount > 0:
                try:
                    LOG.debug(f"Subtracting {t.sold} from {lots[-1]}")
                    capital_gain_delta, exhausted = lots[-1].subtract_sell(t.sold)
                except IndexError:
                    raise RuntimeError(
                        f"Ran out of lots for {currency}. "
                        f"This means more was sold than was ever bought"
                    )
                capital_gain += capital_gain_delta
                if t.sold.amount < 0 or lots[-1].amount < 0:
                    raise RuntimeError(
                        "Somehow buy or sell half went below zero. "
                        "This is a logic error."
                    )
                if exhausted:
                    lots.pop()
    return capital_gain


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
        capital_gain = calculate_capital_gain(transactions, currency)
        LOG.info(f"Capital gain for {currency} is {capital_gain} AUD")


if __name__ == "__main__":
    main()
            
