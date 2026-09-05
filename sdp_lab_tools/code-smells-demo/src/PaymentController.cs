namespace CodeSmellsDemo;

public class PaymentController
{
    private readonly PaymentService _paymentService = new();

    public void HandlePaymentRequest(string paymentType, double amount)
    {
        switch (paymentType)
        {
            case "CASH":
                Console.WriteLine("Routing cash payment to counter.");
                break;
            case "CARD":
                Console.WriteLine("Routing card payment to gateway.");
                break;
            case "BKASH":
                Console.WriteLine("Routing bKash payment to mobile gateway.");
                break;
            case "NAGAD":
                Console.WriteLine("Routing Nagad payment to mobile gateway.");
                break;
            default:
                throw new ArgumentException($"Unknown payment type: {paymentType}");
        }

        _paymentService.Pay(paymentType, amount);
    }
}
