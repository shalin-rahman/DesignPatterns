namespace CodeSmellsDemo;

public class OrderService
{
    private readonly PaymentService _paymentService = new();
    private readonly InvoiceService _invoiceService = new();
    private readonly NotificationService _notificationService = new();
    private readonly DatabaseService _databaseService = new();

    // Long Method: validation, pricing, payment routing, invoicing,
    // notification and persistence are all inlined into one method
    // instead of being split into steps the caller can follow.
    public void ProcessOrder(Order order, string paymentType)
    {
        if (order.Items.Count == 0)
        {
            throw new InvalidOperationException("Order has no items.");
        }

        double subtotal = 0;
        foreach (var item in order.Items)
        {
            if (item.Quantity <= 0)
            {
                throw new InvalidOperationException("Item quantity must be positive.");
            }
            subtotal += item.Price * item.Quantity;
        }

        double discount = order.CalculateDiscount();
        double total = subtotal * (1 - discount);

        string paymentDescription;
        switch (paymentType)
        {
            case "CASH":
                paymentDescription = "cash at counter";
                break;
            case "CARD":
                paymentDescription = "card via gateway";
                break;
            case "BKASH":
                paymentDescription = "bKash mobile transfer";
                break;
            case "NAGAD":
                paymentDescription = "Nagad mobile transfer";
                break;
            default:
                throw new ArgumentException($"Unknown payment type: {paymentType}");
        }

        Console.WriteLine($"Processing order for {order.Customer.Name}: subtotal {subtotal}, discount {discount}, total {total}, via {paymentDescription}");

        _paymentService.Pay(paymentType, total);

        string invoiceLine = _invoiceService.DescribePaymentMethod(paymentType);
        Console.WriteLine($"Invoice: {invoiceLine}, amount {total}");

        _notificationService.NotifyPaymentReceived(paymentType);

        _databaseService.SavePaymentRecord(paymentType, total);

        order.PrintCustomerSnapshot();
    }
}
