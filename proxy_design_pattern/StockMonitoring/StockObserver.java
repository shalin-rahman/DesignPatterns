/**
 * Observer Interface for the Observer Design Pattern.
 * Receives stock price updates without knowing publisher internals.
 */
public interface StockObserver {
    /**
     * Called when a tracked stock symbol changes its price.
     *
     * @param symbol   Ticker symbol
     * @param oldPrice Previous observed price
     * @param newPrice New observed price
     */
    void update(String symbol, double oldPrice, double newPrice);
}
