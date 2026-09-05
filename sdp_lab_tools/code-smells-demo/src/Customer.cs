namespace CodeSmellsDemo;

public class Customer
{
    public string Name = "";
    public int Age;
    public int MembershipYears;
    public int PurchaseHistoryCount;
    public Address Address = new();
    public Account Account = new();
    public Profile Profile = new();
}
