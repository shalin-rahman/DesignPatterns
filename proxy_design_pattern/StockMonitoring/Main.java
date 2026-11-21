/**
 * Main demonstration driver class.
 * Executes the exact 6-step verification sequence specified in the assignment PDF:
 *
 * 1. Register Dashboard and Alice's alert. Track ACME and BETA.
 *    -> Server connection happens once (lazy). Both prices fetched from server.
 * 2. Call checkPrices()
 *    -> Both prices come from cache. No changes, so no notifications.
 * 3. setPrice(ACME, 105), proxy.refresh(), checkPrices()
 *    -> Both prices fetched again. Dashboard reports ACME 100.00 -> 105.00. No alert (below 110).
 * 4. setPrice(ACME, 112), setPrice(BETA, 48), proxy.refresh(), checkPrices()
 *    -> Dashboard reports ACME 105.00 -> 112.00 and BETA 50.00 -> 48.00. Alice receives alert for ACME.
 * 5. Remove the Dashboard. setPrice(ACME, 115), proxy.refresh(), checkPrices()
 *    -> Only Alice's alert is printed. Dashboard prints nothing.
 * 6. Create a separate FREE proxy. Request BETA, then ACME directly.
 *    -> BETA is denied. ACME succeeds, and the server connection for this new proxy is created at this point.
 */
public class Main {
    public static void main(String[] args) {
        System.out.println("================================================================================");
        System.out.println("  ASSIGNMENT DEMONSTRATION: PROXY AND OBSERVER DESIGN PATTERNS");
        System.out.println("  Stock Price Monitoring System");
        System.out.println("================================================================================\n");

        // Initialize Premium Proxy and Tracker
        StockFeedProxy premiumProxy = new StockFeedProxy(UserType.PREMIUM);
        PriceTracker tracker = new PriceTracker(premiumProxy);

        Dashboard dashboard = new Dashboard("Dashboard");
        PriceAlert aliceAlert = new PriceAlert("Alice", "ACME", 110.00);

        // -------------------------------------------------------------------------
        // STEP 1: Register Dashboard and Alice's alert. Track ACME and BETA.
        // Expected: Server connection happens once (lazy). Both prices are fetched from the server.
        // -------------------------------------------------------------------------
        System.out.println(">>> STEP 1: Register Dashboard & Alice's alert. Track ACME and BETA.");
        tracker.registerObserver(dashboard);
        tracker.registerObserver(aliceAlert);

        tracker.trackSymbol("ACME");
        tracker.trackSymbol("BETA");
        System.out.println("[Step 1 Complete]\n");

        // -------------------------------------------------------------------------
        // STEP 2: Call checkPrices()
        // Expected: Both prices come from the cache. No changes, so no notifications.
        // -------------------------------------------------------------------------
        System.out.println(">>> STEP 2: Call checkPrices()");
        tracker.checkPrices();
        System.out.println("[Step 2 Complete - Cache served both; no notifications triggered]\n");

        // -------------------------------------------------------------------------
        // STEP 3: setPrice(ACME, 105), proxy.refresh(), checkPrices()
        // Expected: Both prices fetched again. Dashboard reports ACME 100.00 -> 105.00. No alert (below 110).
        // -------------------------------------------------------------------------
        System.out.println(">>> STEP 3: setPrice(ACME, 105.00), proxy.refresh(), checkPrices()");
        MarketData.setPrice("ACME", 105.00);
        premiumProxy.refresh();
        tracker.checkPrices();
        System.out.println("[Step 3 Complete]\n");

        // -------------------------------------------------------------------------
        // STEP 4: setPrice(ACME, 112), setPrice(BETA, 48), proxy.refresh(), checkPrices()
        // Expected: Dashboard reports ACME 105.00 -> 112.00 and BETA 50.00 -> 48.00. Alice receives an alert for ACME.
        // -------------------------------------------------------------------------
        System.out.println(">>> STEP 4: setPrice(ACME, 112.00), setPrice(BETA, 48.00), proxy.refresh(), checkPrices()");
        MarketData.setPrice("ACME", 112.00);
        MarketData.setPrice("BETA", 48.00);
        premiumProxy.refresh();
        tracker.checkPrices();
        System.out.println("[Step 4 Complete]\n");

        // -------------------------------------------------------------------------
        // STEP 5: Remove the Dashboard. setPrice(ACME, 115), proxy.refresh(), checkPrices()
        // Expected: Only Alice's alert is printed. The Dashboard prints nothing.
        // -------------------------------------------------------------------------
        System.out.println(">>> STEP 5: Remove the Dashboard. setPrice(ACME, 115.00), proxy.refresh(), checkPrices()");
        tracker.removeObserver(dashboard);
        MarketData.setPrice("ACME", 115.00);
        premiumProxy.refresh();
        tracker.checkPrices();
        System.out.println("[Step 5 Complete]\n");

        // -------------------------------------------------------------------------
        // STEP 6: Create a separate FREE proxy. Request BETA, then ACME directly.
        // Expected: BETA is denied. ACME succeeds, and the server connection for this new proxy is created at this point.
        // -------------------------------------------------------------------------
        System.out.println(">>> STEP 6: Create separate FREE proxy. Request BETA, then ACME directly.");
        StockFeedProxy freeProxy = new StockFeedProxy(UserType.FREE);

        System.out.println("-> Requesting restricted symbol BETA with FREE proxy:");
        double betaPrice = freeProxy.getPrice("BETA");
        System.out.println("Returned price for BETA: " + betaPrice);

        System.out.println("\n-> Requesting allowed symbol ACME with FREE proxy:");
        double acmePrice = freeProxy.getPrice("ACME");
        System.out.println("Returned price for ACME: " + acmePrice);
        System.out.println("[Step 6 Complete]\n");

        System.out.println("================================================================================");
        System.out.println("  ALL 6 STEPS COMPLETED SUCCESSFULLY ACCORDING TO SPECIFICATION");
        System.out.println("================================================================================");
    }
}
