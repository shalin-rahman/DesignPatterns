import java.util.HashMap;
import java.util.Map;

/**
 * Proxy in the Proxy Pattern.
 * Implements StockFeed and manages access to RealStockFeed with:
 * 1. Lazy Initialization: RealStockFeed is instantiated only upon first valid request.
 * 2. Caching: Caches fetched stock prices; returns cached values until refresh() is called.
 * 3. Access Control (Protection Proxy): Restricts premium-only symbols (e.g. BETA) from FREE users.
 */
public class StockFeedProxy implements StockFeed {
    private RealStockFeed realFeed;
    private final UserType userType;
    private final Map<String, Double> cache;

    public StockFeedProxy(UserType userType) {
        this.userType = userType;
        this.cache = new HashMap<>();
        this.realFeed = null; // Lazy connection
    }

    @Override
    public double getPrice(String symbol) {
        // 1. Access Control Check
        if ("BETA".equalsIgnoreCase(symbol) && userType == UserType.FREE) {
            System.out.println("[StockFeedProxy] ACCESS DENIED: Symbol '" + symbol + "' is restricted to PREMIUM users.");
            return -1.0;
        }

        // 2. Check Cache
        if (cache.containsKey(symbol)) {
            System.out.println("[StockFeedProxy] Cache HIT: Returning cached price for " + symbol + " (" + cache.get(symbol) + ")");
            return cache.get(symbol);
        }

        // 3. Lazy Initialization of RealSubject
        if (realFeed == null) {
            System.out.println("[StockFeedProxy] Initializing RealStockFeed connection lazily...");
            realFeed = new RealStockFeed();
        }

        // 4. Fetch from RealSubject & Store in Cache
        double price = realFeed.getPrice(symbol);
        cache.put(symbol, price);
        return price;
    }

    /**
     * Clears cached prices, forcing subsequent requests to fetch fresh prices from the server.
     */
    public void refresh() {
        System.out.println("[StockFeedProxy] Refreshing cache. All cached prices cleared.");
        cache.clear();
    }

    public UserType getUserType() {
        return userType;
    }
}
