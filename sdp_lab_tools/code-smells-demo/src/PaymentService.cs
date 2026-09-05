namespace CodeSmellsDemo;

public class PaymentService
{
    // Switch Statements + Shotgun Surgery: adding a new payment type means
    // touching this switch, plus the matching switch in PaymentController,
    // InvoiceService, NotificationService, DatabaseService and OrderService.
    public void Pay(string paymentType, double amount)
    {
        switch (paymentType)
        {
            case "CASH":
                Console.WriteLine($"Paid {amount} in cash.");
                break;
            case "CARD":
                Console.WriteLine($"Charged {amount} to card.");
                break;
            case "BKASH":
                Console.WriteLine($"Paid {amount} via bKash.");
                break;
            case "NAGAD":
                Console.WriteLine($"Paid {amount} via Nagad.");
                break;
            default:
                throw new ArgumentException($"Unknown payment type: {paymentType}");
        }
    }
}
