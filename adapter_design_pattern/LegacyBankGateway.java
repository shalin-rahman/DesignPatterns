/**
 * Adaptee 1: LegacyBankGateway
 * 
 * An older, existing banking gateway provided by an external vendor.
 * Notice its quirks:
 * 1. It only understands cents (not dollars).
 * 2. It returns integer error codes (0 for success, non-zero for failure).
 * 3. It rejects any single transfer larger than 100,000 cents ($1,000.00).
 * 
 * Per the assignment rules, we CANNOT change anything in this class.
 */
public class LegacyBankGateway {

    /**
     * Charges an account in whole cents.
     * Returns 0 on success, or 1 if declined/over limit.
     */
    public int makeTransfer(String accountHolder, long amountInCents) {
        System.out.printf("  [LegacyBankGateway] Processing transfer: %d cents for %s...%n", amountInCents, accountHolder);
        
        // The bank declines any transfer higher than 100,000 cents ($1000)
        if (amountInCents > 100000) {
            System.out.printf("  [LegacyBankGateway] Transfer declined: %d cents exceeds limit of 100000 cents.%n", amountInCents);
            return 1; // Failure status code
        }
        
        System.out.printf("  [LegacyBankGateway] Transfer approved: %d cents for %s.%n", amountInCents, accountHolder);
        return 0; // Success status code
    }

    /**
     * Reverses a previous transfer in whole cents.
     */
    public int reverseTransfer(String accountHolder, long amountInCents) {
        System.out.printf("  [LegacyBankGateway] Processing reverse transfer: %d cents for %s...%n", amountInCents, accountHolder);
        System.out.printf("  [LegacyBankGateway] Reverse transfer approved: %d cents for %s.%n", amountInCents, accountHolder);
        return 0; // Success status code
    }
}
