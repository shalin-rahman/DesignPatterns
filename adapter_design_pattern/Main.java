import java.util.ArrayList;
import java.util.List;

/**
 * Driver: Main
 * 
 * This is the entry point that demonstrates the full Adapter pattern in action.
 * 
 * The structure is simple:
 *   1. Create the third-party Adaptee objects (the raw SDKs/services).
 *   2. Wrap each Adaptee in its matching Adapter.
 *   3. Hand everything to Checkout (the client) — which never sees the Adaptees at all.
 * 
 * You can add a fourth payment service simply by creating a new Adapter class and
 * passing it to checkout.pay() or checkout.refund() — without touching Checkout or
 * any existing Adapter.
 */
public class Main {
    public static void main(String[] args) {
        System.out.println("==========================================================");
        System.out.println("     Online Store Payment System (Adapter Pattern Demo)   ");
        System.out.println("==========================================================\n");

        // -----------------------------------------------------------------------
        // Step 1: Create the Adaptees
        //   These are the raw third-party services with their own incompatible APIs.
        //   In real life these would come from an external JAR or SDK — we can't
        //   change their code at all.
        // -----------------------------------------------------------------------
        LegacyBankGateway bankGateway = new LegacyBankGateway();   // works in cents, returns int codes
        QuickPaySDK       quickPaySDK = new QuickPaySDK();          // needs currency string, returns txId
        EuroPayService  euroPayService = new EuroPayService();      // charges in EUR, needs email

        // -----------------------------------------------------------------------
        // Step 2: Wrap each Adaptee in its matching Adapter
        //   After this point, everyone speaks the same PaymentProcessor language.
        // -----------------------------------------------------------------------
        PaymentProcessor legacyBank = new LegacyBankAdapter(bankGateway);   // converts cents/status codes
        PaymentProcessor quickPay   = new QuickPayAdapter(quickPaySDK);     // caches txIds, adds currency
        PaymentProcessor euroPay    = new EuroPayAdapter(euroPayService);   // converts USD->EUR, builds email

        // Storing different payment adapters in a single list proves they're interchangeable
        List<PaymentProcessor> processors = new ArrayList<>();
        processors.add(legacyBank);
        processors.add(quickPay);
        processors.add(euroPay);

        System.out.println("Configured Payment Processors:");
        for (PaymentProcessor p : processors) {
            System.out.println(" - " + p.getName());
        }
        System.out.println();

        // -----------------------------------------------------------------------
        // Step 3: Let Checkout handle all transactions via the common interface
        //   Notice Checkout never imports or mentions LegacyBankGateway, QuickPaySDK,
        //   or EuroPayService — it doesn't need to know they exist.
        // -----------------------------------------------------------------------
        Checkout checkout = new Checkout();

        System.out.println("--- Executing Sample Transactions ---");

        // TX 1: Legacy Bank — $25.50 -> converted to 2550 cents internally by the adapter
        checkout.pay(legacyBank, "Alice", 25.50);

        // TX 2: QuickPay — $100.00 -> adapter adds "USD" and saves txId "QP-1001" for Bob
        checkout.pay(quickPay, "Bob", 100.00);

        // TX 3: EuroPay — $50.00 * 0.90 = 45.00 EUR; "Carol" becomes "carol@store.com"
        checkout.pay(euroPay, "Carol", 50.00);

        // TX 4: Legacy Bank — $1500 = 150,000 cents > bank limit of 100,000 cents -> FAILED
        checkout.pay(legacyBank, "Dave", 1500.00);

        // TX 5: QuickPay refund for Bob — adapter looks up Bob's saved txId ("QP-1001") automatically
        checkout.refund(quickPay, "Bob", 100.00);

        // TX 6: EuroPay refund for Carol — $50.00 reconverted to 45.00 EUR, email rebuilt
        checkout.refund(euroPay, "Carol", 50.00);

        System.out.println("\nAll transactions completed.");
    }
}
