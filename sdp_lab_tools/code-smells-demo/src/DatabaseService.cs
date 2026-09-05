namespace CodeSmellsDemo;

public class DatabaseService
{
    public void SavePaymentRecord(string paymentType, double amount)
    {
        switch (paymentType)
        {
            case "CASH":
                Console.WriteLine($"INSERT INTO cash_payments VALUES ({amount})");
                break;
            case "CARD":
                Console.WriteLine($"INSERT INTO card_payments VALUES ({amount})");
                break;
            case "BKASH":
                Console.WriteLine($"INSERT INTO mobile_payments VALUES ('BKASH', {amount})");
                break;
            case "NAGAD":
                Console.WriteLine($"INSERT INTO mobile_payments VALUES ('NAGAD', {amount})");
                break;
            default:
                throw new ArgumentException($"Unknown payment type: {paymentType}");
        }
    }
}
