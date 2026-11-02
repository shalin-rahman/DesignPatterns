import java.util.HashMap;
import java.util.Map;

/**
 * Adapter 2: QuickPayAdapter
 * 
 * What this adapter does:
 * It connects our store's PaymentProcessor interface with the QuickPaySDK.
 * 
 * Key conversions and state management handled here:
 * 1. Parameter reordering & currency: QuickPay requires (currency, amount, payerName).
 *    Our interface provides (customer, amount). The adapter rearranges the arguments
 *    and supplies the required "USD" currency string.
 * 2. Return value translation: QuickPay returns a transaction ID string (e.g. "QP-1001")
 *    or null. We convert non-null into true (success) and null into false (failure).
 * 3. Remembering transaction IDs for refunds:
 *    This is the tricky part! When a refund is requested, our client only gives us
 *    (customer, amount). But QuickPaySDK.returnMoney() expects a transaction ID!
 *    To solve this, our adapter keeps an internal cache (a Map) linking customer names
 *    to their latest transaction ID. When refund is called, we look up that transaction ID
 *    and pass it to QuickPay.
 */
public class QuickPayAdapter implements PaymentProcessor {

    // Reference to the third-party QuickPay SDK
    private final QuickPaySDK quickPaySDK;

    // Internal memory: maps customer name -> last transaction ID (e.g. "Bob" -> "QP-1001")
    private final Map<String, String> customerTransactions;

    public QuickPayAdapter(QuickPaySDK quickPaySDK) {
        this.quickPaySDK = quickPaySDK;
        this.customerTransactions = new HashMap<>();
    }

    @Override
    public String getName() {
        return "QuickPay";
    }

    @Override
    public boolean pay(String customer, double amountInDollars) {
        // QuickPay expects: currency ("USD"), amount, and payerName
        String transactionId = quickPaySDK.sendMoney("USD", amountInDollars, customer);

        // If a valid transaction ID came back, the payment succeeded
        if (transactionId != null) {
            // Save the transaction ID so we can use it later if a refund is requested!
            customerTransactions.put(customer, transactionId);
            return true;
        }

        return false;
    }

    @Override
    public boolean refund(String customer, double amountInDollars) {
        // The store asks for a refund for "customer", but QuickPay needs a transaction ID.
        // We retrieve the transaction ID we saved when this customer paid earlier.
        String transactionId = customerTransactions.get(customer);

        if (transactionId == null) {
            System.out.printf("  [QuickPayAdapter] Cannot refund: No previous transaction found for customer '%s'.%n", customer);
            return false;
        }

        // Delegate to the SDK using the remembered transaction ID
        return quickPaySDK.returnMoney(transactionId);
    }
}
