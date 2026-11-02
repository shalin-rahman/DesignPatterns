/**
 * Target Interface: PaymentProcessor
 * 
 * This is the contract our checkout system expects every payment service to follow.
 * The client (Checkout) only talks to this interface and has no idea which actual
 * third-party service is doing the heavy lifting behind the scenes.
 */
public interface PaymentProcessor {

    /**
     * Human-readable display name for the payment method (e.g. "Legacy Bank").
     */
    String getName();

    /**
     * Processes a payment in US Dollars.
     *
     * @param customer        Name of the customer making the purchase.
     * @param amountInDollars Total amount to charge in USD.
     * @return true if the payment succeeded, false otherwise.
     */
    boolean pay(String customer, double amountInDollars);

    /**
     * Issues a refund in US Dollars.
     *
     * @param customer        Name of the customer receiving the refund.
     * @param amountInDollars Total amount to refund in USD.
     * @return true if the refund succeeded, false otherwise.
     */
    boolean refund(String customer, double amountInDollars);
}
