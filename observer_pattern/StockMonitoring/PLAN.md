# Implementation Plan - Stock Price Monitoring System (Proxy & Observer Patterns)

## Context & Objectives
Implement the complete assignment from `Assignment Proxy and Observer Design Patterns.pdf`:
- Both patterns working together:
  - **Proxy Pattern**: `StockFeed` (Subject), `RealStockFeed` (Real Subject), `StockFeedProxy` (Proxy with lazy loading, caching + refresh, access control for PREMIUM/FREE with BETA restricted), and `UserType` enum.
  - **Observer Pattern**: `StockObserver` (Observer interface with `onPriceChanged(String symbol, double oldPrice, double newPrice)`), `PriceTracker` (Subject / Observable with `registerObserver`, `removeObserver`, `notifyObservers`, `trackSymbol`, `checkPrices`), `Dashboard` (Concrete Observer), `PriceAlert` (Concrete Observer for investor with threshold target).
  - **Environment Simulation**: `MarketData` (static symbol-price store).
  - **Client / Driver**: `Main` implementing exact 6-step verification sequence from PDF.
- Place implementation in `observer_pattern/StockMonitoring` and `proxy_design_pattern/StockMonitoring` (or linked/co-located as appropriate with documentation, Task 1 answers, and Mermaid diagrams).

## Proposed Architecture & Structure
Files to create in `observer_pattern/StockMonitoring` and `proxy_design_pattern/StockMonitoring`:
1. `UserType.java`: enum `PREMIUM`, `FREE`.
2. `MarketData.java`: static store for prices (`setPrice`, `getPrice`).
3. `StockFeed.java`: interface with `double getPrice(String symbol)`.
4. `RealStockFeed.java`: connects to server lazily, logs connection & contact, fetches from `MarketData`.
5. `StockFeedProxy.java`: implements `StockFeed`, controls access (BETA for PREMIUM only), caches prices, connects lazily, offers `refresh()`.
6. `StockObserver.java`: observer interface receiving `(String symbol, double oldPrice, double newPrice)`.
7. `Dashboard.java`: prints every price change reported.
8. `PriceAlert.java`: prints alert only when its symbol's price reaches or exceeds target price.
9. `PriceTracker.java`: observable watching tracked symbols via `StockFeed`, tracks previous prices, notifies registered observers when price changes on `checkPrices()`.
10. `Main.java`: executes exact steps 1-6 from sample demonstration sequence.
11. `README.md`: Deliverable Task 1 (pattern elements table and rationale), Task 2 (Mermaid class and sequence diagrams), and demonstration output.

## Verification Plan
1. Compile using `javac *.java`.
2. Run `java Main` and verify exact match with expected behavior in Steps 1 to 6.
