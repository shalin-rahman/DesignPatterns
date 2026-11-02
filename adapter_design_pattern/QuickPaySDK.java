/**
 * Adaptee 2: QuickPaySDK
 * 
 * A modern SDK from a fintech company with its own unique conventions:
 * 1. Method parameter order is (currency, amount, payerName) instead of (customer, amount).
 * 2. It requires an explicit currency string like "USD".
 * 3. Payment returns a unique String transaction ID (e.g. "QP-1001") on success, or null on failure.
 * 4. Most importantly: its refund method ONLY takes a transaction ID, NOT a customer name!
 * 
 * We cannot change this third-party SDK, so our adapter will have to bridge these differences.
 */
public class QuickPaySDK {
    // Generates simple ascending IDs like QP-1001, QP-1002, etc.
    private int transactionCounter = 1001;

    /**
     * Sends money through QuickPay.
     * Returns the assigned transaction ID string on success, or null on failure.
     */
    public String sendMoney(String currency, double amount, String payerName) {
        System.out.printf("  [QuickPaySDK] Sending %.2f %s for payer '%s'...%n", amount, currency, payerName);
        
        if (amount <= 0 || currency == null || payerName == null) {
            System.out.println("  [QuickPaySDK] Payment failed: invalid transaction parameters.");
            return null;
        }
        
        String transactionId = "QP-" + transactionCounter++;
        System.out.printf("  [QuickPaySDK] Payment succeeded. Assigned Transaction ID: %s%n", transactionId);
        return transactionId;
    }

    /**
     * Refunds a past payment using its original transaction ID.
     */
    public boolean returnMoney(String transactionId) {
        System.out.printf("  [QuickPaySDK] Processing refund for Transaction ID: %s...%n", transactionId);
        
        if (transactionId == null || !transactionId.startsWith("QP-")) {
            System.out.printf("  [QuickPaySDK] Refund failed: unrecognized Transaction ID '%s'.%n", transactionId);
            return false;
        }
        
        System.out.printf("  [QuickPaySDK] Refund completed for Transaction ID: %s.%n", transactionId);
        return true;
    }
}
