/**
 * Subject Interface for the Proxy Design Pattern.
 * Common abstraction for both RealStockFeed and StockFeedProxy.
 */
public interface StockFeed {
    /**
     * Returns the current price of a stock symbol.
     *
     * @param symbol Ticker symbol (e.g., ACME, BETA)
     * @return current price
     */
    double getPrice(String symbol);
}
