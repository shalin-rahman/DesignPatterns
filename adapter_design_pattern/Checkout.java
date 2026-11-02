import java.util.Locale;

/**
 * Client: Checkout
 * 
 * Notice what is MISSING from this class:
 * - NO "if (processor instanceof LegacyBankAdapter)"
 * - NO checks for currencies, emails, transaction IDs, or status codes.
 * - NO vendor-specific imports.
 * 
 * The checkout system only cares about the PaymentProcessor interface.
 * Whether the user pays with a legacy bank in cents, QuickPay with transaction IDs,
 * or EuroPay in euros, Checkout calls the exact same pay() and refund() methods
 * and prints a clean, uniform receipt.
 */
public class Checkout {

    /**
     * Executes a payment for any payment processor and prints the outcome.
     */
    public boolean pay(PaymentProcessor processor, String customer, double amountInDollars) {
        // Clean polymorphism: call pay() regardless of which service is behind the interface
        boolean success = processor.pay(customer, amountInDollars);
        
        // Print standardized receipt
        printReceipt(processor.getName(), "pay", customer, amountInDollars, success);
        return success;
    }

    /**
     * Executes a refund for any payment processor and prints the outcome.
     */
    public boolean refund(PaymentProcessor processor, String customer, double amountInDollars) {
        // Clean polymorphism: call refund() regardless of which service is behind the interface
        boolean success = processor.refund(customer, amountInDollars);
        
        // Print standardized receipt
        printReceipt(processor.getName(), "refund", customer, amountInDollars, success);
        return success;
    }

    /**
     * Formats and prints the receipt in the exact layout required by the assignment:
     * Legacy Bank | pay | Alice | $25.50 -> SUCCESS
     */
    private void printReceipt(String serviceName, String action, String customer, double amount, boolean success) {
        String result = success ? "SUCCESS" : "FAILED";
        System.out.printf(Locale.US, "%s | %s | %s | $%.2f -> %s%n", serviceName, action, customer, amount, result);
    }
}
