/**
 * RealSubject in the Proxy Pattern.
 * Simulates a slow, expensive remote connection to the stock exchange server.
 */
public class RealStockFeed implements StockFeed {

    public RealStockFeed() {
        // Simulate slow connection setup
        connectToServer();
    }

    private void connectToServer() {
        System.out.println("[RealStockFeed] Establishing slow and expensive connection to remote stock server...");
    }

    @Override
    public double getPrice(String symbol) {
        System.out.println("[RealStockFeed] Contacting remote server for price of " + symbol + "...");
        return MarketData.getPrice(symbol);
    }
}
