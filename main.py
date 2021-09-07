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
    amount: float
    unit: str
    

@dataclass
class Transaction:
    """ One full transaction. Assumes that one half of the transaction is in AUD. """
    # Whatever was sold in the transaction.
    sold: Half
    # What was aquired in the transaction.
    bought: Half
    # Rate. Don't use this field directly.
    rate: float

    @classmethod
    def from_rows(cls, first_row, second_row):
        first_is_debit = bool(first_row["debit"])
        if first_is_debit:
            first_amount = first_row["debit"]
            second_amount = second_row["credit"]
        else:
            first_amount = first_row["credit"]
            second_amount = second_row["debit"]
        first = Half(amount=first_amount, unit=first_row["currency"])
        second = Half(amount=second_amount, unit=second_row["currency"])
        if first_is_debit:
            sold = first
            bought = second
        else:
            sold = second
            bought = first
        rate = float(first_row["rates"].split("$")[1].split(" ")[0].replace(",", ""))
        return Transaction(sold=sold, bought=bought, rate=rate)



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
    pass


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



if __name__ == "__main__":
    main()
            
