namespace CodeSmellsDemo;

public class NotificationService
{
    public void NotifyPaymentReceived(string paymentType)
    {
        switch (paymentType)
        {
            case "CASH":
                Console.WriteLine("SMS: cash payment received.");
                break;
            case "CARD":
                Console.WriteLine("Email: card payment received.");
                break;
            case "BKASH":
                Console.WriteLine("SMS: bKash payment received.");
                break;
            case "NAGAD":
                Console.WriteLine("SMS: Nagad payment received.");
                break;
            default:
                throw new ArgumentException($"Unknown payment type: {paymentType}");
        }
    }
}
