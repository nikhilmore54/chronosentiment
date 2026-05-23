import yfinance as yf
from scripts.candle_substrate import ProviderTelemetrySession
import sys

session = ProviderTelemetrySession()
df = yf.download("AAPL", period="1d", interval="5m", progress=False, session=session)
print(df.shape)
print(session.flush_metrics())
