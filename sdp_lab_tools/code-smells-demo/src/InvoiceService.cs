namespace CodeSmellsDemo;

public class InvoiceService
{
    public string DescribePaymentMethod(string paymentType)
    {
        switch (paymentType)
        {
            case "CASH":
                return "Paid in cash";
            case "CARD":
                return "Paid by card";
            case "BKASH":
                return "Paid via bKash";
            case "NAGAD":
                return "Paid via Nagad";
            default:
                throw new ArgumentException($"Unknown payment type: {paymentType}");
        }
    }
}
