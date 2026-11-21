import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

/**
 * Subject (Observable) in the Observer Pattern and Client to StockFeed (Proxy).
 * Watches stock symbols using a StockFeed abstraction.
 * Upon checkPrices(), checks for price differences and notifies registered observers.
 */
public class PriceTracker {
    private final StockFeed stockFeed;
    private final List<StockObserver> observers;
    private final List<String> trackedSymbols;
    private final Map<String, Double> lastPrices;

    public PriceTracker(StockFeed stockFeed) {
        this.stockFeed = stockFeed;
        this.observers = new ArrayList<>();
        this.trackedSymbols = new ArrayList<>();
        this.lastPrices = new HashMap<>();
    }

    public void registerObserver(StockObserver observer) {
        if (observer != null && !observers.contains(observer)) {
            observers.add(observer);
        }
    }

    public void removeObserver(StockObserver observer) {
        observers.remove(observer);
    }

    public void trackSymbol(String symbol) {
        if (!trackedSymbols.contains(symbol)) {
            trackedSymbols.add(symbol);
            // Fetch initial baseline price upon starting tracking
            double initialPrice = stockFeed.getPrice(symbol);
            if (initialPrice >= 0) {
                lastPrices.put(symbol, initialPrice);
            }
        }
    }

    public void untrackSymbol(String symbol) {
        trackedSymbols.remove(symbol);
        lastPrices.remove(symbol);
    }

    /**
     * Checks prices for all tracked symbols.
     * If a price has changed since the previous check, notifies all registered observers.
     */
    public void checkPrices() {
        for (String symbol : trackedSymbols) {
            double currentPrice = stockFeed.getPrice(symbol);
            if (currentPrice < 0) {
                // Access denied or invalid symbol
                continue;
            }

            Double oldPrice = lastPrices.get(symbol);
            if (oldPrice == null) {
                lastPrices.put(symbol, currentPrice);
            } else if (Double.compare(oldPrice, currentPrice) != 0) {
                lastPrices.put(symbol, currentPrice);
                notifyObservers(symbol, oldPrice, currentPrice);
            }
        }
    }

    private void notifyObservers(String symbol, double oldPrice, double newPrice) {
        for (StockObserver observer : observers) {
            observer.update(symbol, oldPrice, newPrice);
        }
    }
}
