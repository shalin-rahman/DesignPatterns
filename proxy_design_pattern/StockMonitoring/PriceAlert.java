/**
 * Concrete Observer in the Observer Pattern.
 * Belongs to an investor and monitors a specific symbol with a target price.
 * Alerts only when the new price is at or above the target threshold.
 */
public class PriceAlert implements StockObserver {
    private final String investorName;
    private final String targetSymbol;
    private final double targetPrice;

    public PriceAlert(String investorName, String targetSymbol, double targetPrice) {
        this.investorName = investorName;
        this.targetSymbol = targetSymbol;
        this.targetPrice = targetPrice;
    }

    @Override
    public void update(String symbol, double oldPrice, double newPrice) {
        // Only inspect updates for matching symbol, without any instanceof or type casting
        if (targetSymbol.equalsIgnoreCase(symbol) && newPrice >= targetPrice) {
            System.out.printf("[PriceAlert for %s] ALERT! %s reached target price %.2f (Current: %.2f, Previous: %.2f)%n",
                    investorName, symbol, targetPrice, newPrice, oldPrice);
        }
    }

    public String getInvestorName() {
        return investorName;
    }

    public String getTargetSymbol() {
        return targetSymbol;
    }

    public double getTargetPrice() {
        return targetPrice;
    }
}
