/**
 * Adapter 1: LegacyBankAdapter
 * 
 * What this adapter does:
 * It acts as a bridge between our store's standard PaymentProcessor interface
 * and the third-party LegacyBankGateway.
 * 
 * Key conversions handled here:
 * 1. Unit conversion: The store works in dollars ($25.50), but the bank only
 *    accepts whole cents (2550 cents). We multiply by 100 and round cleanly.
 * 2. Return type translation: The bank returns an integer status code (0 = success,
 *    anything else = error). We convert that into a simple true/false boolean
 *    that our checkout system expects.
 */
public class LegacyBankAdapter implements PaymentProcessor {
    
    // We hold a reference to the actual bank gateway (Object Adapter pattern using composition)
    private final LegacyBankGateway gateway;

    public LegacyBankAdapter(LegacyBankGateway gateway) {
        this.gateway = gateway;
    }

    @Override
    public String getName() {
        return "Legacy Bank";
    }

    @Override
    public boolean pay(String customer, double amountInDollars) {
        // Step 1: Convert dollar amount to cents (e.g. $25.50 -> 2550 cents)
        long amountInCents = Math.round(amountInDollars * 100);

        // Step 2: Call the bank's makeTransfer method
        int statusCode = gateway.makeTransfer(customer, amountInCents);

        // Step 3: Bank gives 0 for success; convert that to boolean true
        return statusCode == 0;
    }

    @Override
    public boolean refund(String customer, double amountInDollars) {
        // Step 1: Convert dollar amount to cents
        long amountInCents = Math.round(amountInDollars * 100);

        // Step 2: Call the bank's reverseTransfer method
        int statusCode = gateway.reverseTransfer(customer, amountInCents);

        // Step 3: Convert bank status code to boolean true/false
        return statusCode == 0;
    }
}
