/**
 * Adaptee 3: EuroPayService
 * 
 * A European payment provider with its own specific rules:
 * 1. Only operates in Euros (EUR), not US Dollars.
 * 2. Identifies customers by email address rather than just their name.
 * 
 * We cannot change this third-party code. Our adapter will handle currency
 * conversion and email construction.
 */
public class EuroPayService {

    /**
     * Charges a customer's card in Euros using their registered email.
     */
    public boolean chargeCard(String customerEmail, double amountInEuros) {
        System.out.printf("  [EuroPayService] Charging %.2f EUR to account %s...%n", amountInEuros, customerEmail);
        
        if (amountInEuros <= 0 || customerEmail == null || !customerEmail.contains("@")) {
            System.out.println("  [EuroPayService] Charge declined: invalid email or amount.");
            return false;
        }
        
        System.out.printf("  [EuroPayService] Charge successful: %.2f EUR from %s.%n", amountInEuros, customerEmail);
        return true;
    }

    /**
     * Refunds a customer's card in Euros using their registered email.
     */
    public boolean refundCard(String customerEmail, double amountInEuros) {
        System.out.printf("  [EuroPayService] Refunding %.2f EUR to account %s...%n", amountInEuros, customerEmail);
        
        if (amountInEuros <= 0 || customerEmail == null || !customerEmail.contains("@")) {
            System.out.println("  [EuroPayService] Refund failed: invalid email or amount.");
            return false;
        }
        
        System.out.printf("  [EuroPayService] Refund successful: %.2f EUR to %s.%n", amountInEuros, customerEmail);
        return true;
    }
}
