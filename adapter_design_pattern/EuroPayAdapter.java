/**
 * Adapter 3: EuroPayAdapter
 * 
 * What this adapter does:
 * It bridges our store's PaymentProcessor interface with EuroPayService.
 * 
 * Key conversions handled here:
 * 1. Customer identifier mapping: The store identifies people by name ("Carol"),
 *    but EuroPay requires an email address. We convert "Carol" into "carol@store.com".
 * 2. Currency exchange: The store bills in US Dollars ($50.00), but EuroPay only
 *    charges in Euros. We multiply by the fixed rate of 0.90 ($50.00 * 0.90 = 45.00 EUR).
 */
public class EuroPayAdapter implements PaymentProcessor {

    private final EuroPayService euroPayService;

    // Fixed store exchange rate: 1 USD = 0.90 EUR
    private static final double USD_TO_EUR_RATE = 0.90;

    public EuroPayAdapter(EuroPayService euroPayService) {
        this.euroPayService = euroPayService;
    }

    @Override
    public String getName() {
        return "EuroPay";
    }

    @Override
    public boolean pay(String customer, double amountInDollars) {
        // Step 1: Format customer name into store email (e.g. "Carol" -> "carol@store.com")
        String email = buildEmail(customer);

        // Step 2: Convert dollars to euros (e.g. $50.00 * 0.90 = 45.00 EUR)
        double euros = convertToEuros(amountInDollars);

        // Step 3: Delegate the charge to the EuroPay service
        return euroPayService.chargeCard(email, euros);
    }

    @Override
    public boolean refund(String customer, double amountInDollars) {
        // Step 1: Format email
        String email = buildEmail(customer);

        // Step 2: Convert dollars to euros
        double euros = convertToEuros(amountInDollars);

        // Step 3: Delegate the refund to the EuroPay service
        return euroPayService.refundCard(email, euros);
    }

    /**
     * Helper to create a consistent lowercase store email from the customer name.
     */
    private String buildEmail(String customer) {
        return customer.trim().toLowerCase() + "@store.com";
    }

    /**
     * Helper to convert US Dollars to Euros using the fixed conversion rate.
     */
    private double convertToEuros(double amountInDollars) {
        return amountInDollars * USD_TO_EUR_RATE;
    }
}
