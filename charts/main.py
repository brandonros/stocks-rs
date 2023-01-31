import sqlite3
import pandas
import matplotlib.pyplot as pyplot
import datetime

connection = sqlite3.connect("../database.db")
sql = """select scraped_at, last_trade_price, (last_trade_price - LAG(last_trade_price, 1) over (order by scraped_at asc)) as price_difference, (scraped_at - LAG(scraped_at, 1) over (order by scraped_at asc)) as age_difference from quote_snapshots where scraped_at >= 1675175400"""
rows = pandas.read_sql(sql, connection)
rows['scraped_at'] = pandas.to_datetime(rows['scraped_at'], unit='s')

#pyplot.plot(rows.scraped_at, rows.age_difference, label = "age_difference")
#pyplot.plot(rows.scraped_at, rows.price_difference, label = "price_difference")
pyplot.plot(rows.scraped_at, rows.last_trade_price, label = "last_trade_price")

pyplot.legend()
pyplot.title("quote_snapshots")
pyplot.show()
