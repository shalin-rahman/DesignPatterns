namespace CodeSmellsDemo;

public class Order
{
    public Customer Customer = new();
    public List<Item> Items = new();

    // Feature Envy: this method cares more about Customer's data than
    // about the Order itself — almost every line reaches into Customer.
    public double CalculateDiscount()
    {
        double discount = 0;

        if (Customer.Age > 60)
        {
            discount += 0.05;
        }

        if (Customer.MembershipYears > 5)
        {
            discount += 0.10;
        }
        else if (Customer.MembershipYears > 1)
        {
            discount += 0.03;
        }

        if (Customer.PurchaseHistoryCount > 50)
        {
            discount += 0.07;
        }

        if (Customer.Account.Balance > 1000)
        {
            discount += 0.02;
        }

        return discount;
    }

    // Inappropriate Intimacy: reaches three levels into Customer's
    // internals (Address, Account, Profile.Membership) instead of asking
    // Customer to describe itself.
    public void PrintCustomerSnapshot()
    {
        Console.WriteLine($"Customer: {Customer.Name}");
        Console.WriteLine($"City: {Customer.Address.City}");
        Console.WriteLine($"Balance: {Customer.Account.Balance}");
        Console.WriteLine($"Membership type: {Customer.Profile.Membership.Type}");
    }

    // Data Clumps: street/city/postalCode/country always travel together
    // as four separate parameters instead of one Address value.
    public void ShipTo(string street, string city, string postalCode, string country)
    {
        Console.WriteLine($"Shipping to {street}, {city}, {postalCode}, {country}");
    }

    public void BillTo(string street, string city, string postalCode, string country)
    {
        Console.WriteLine($"Billing to {street}, {city}, {postalCode}, {country}");
    }
}
