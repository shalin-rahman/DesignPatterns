import java.util.HashMap;
import java.util.Map;

/**
 * Simulates the remote stock market database.
 * Holds current prices for stock symbols.
 */
public class MarketData {
    private static final Map<String, Double> prices = new HashMap<>();

    static {
        // Initial market prices specified in assignment
        prices.put("ACME", 100.00);
        prices.put("BETA", 50.00);
    }

    /**
     * Updates or sets the current price of a stock symbol.
     * Used by Main / Simulation drivers to move the market.
     */
    public static void setPrice(String symbol, double price) {
        prices.put(symbol, price);
    }

    /**
     * Fetches the current price of a stock symbol from the simulated market.
     */
    public static double getPrice(String symbol) {
        if (!prices.containsKey(symbol)) {
            throw new IllegalArgumentException("Unknown stock symbol: " + symbol);
        }
        return prices.get(symbol);
    }
}
