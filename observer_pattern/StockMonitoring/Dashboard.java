/**
 * Concrete Observer in the Observer Pattern.
 * Prints every price change it receives.
 */
public class Dashboard implements StockObserver {
    private final String name;

    public Dashboard(String name) {
        this.name = name;
    }

    public Dashboard() {
        this("Main Dashboard");
    }

    @Override
    public void update(String symbol, double oldPrice, double newPrice) {
        System.out.printf("[%s] Notice: %s price changed: %.2f -> %.2f%n",
                name, symbol, oldPrice, newPrice);
    }
}
