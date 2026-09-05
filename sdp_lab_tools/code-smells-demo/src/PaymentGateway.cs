namespace CodeSmellsDemo;

// Speculative Generality: three layers of abstraction built "for the future"
// — a second gateway provider, or a factory-of-factories — that never
// arrives. Only CardPaymentGateway exists, and the app only ever asks for
// exactly that one concrete type, so none of the flexibility below is used.

public interface IPaymentGateway
{
    void Charge(double amount);
}

public interface IPaymentGatewayFactory
{
    IPaymentGateway Create();
}

public abstract class AbstractPaymentGatewayFactoryProvider
{
    public abstract IPaymentGatewayFactory GetFactory();
}

public class CardPaymentGateway : IPaymentGateway
{
    public void Charge(double amount)
    {
        Console.WriteLine($"Card gateway charged {amount}.");
    }
}

public class CardPaymentGatewayFactory : IPaymentGatewayFactory
{
    public IPaymentGateway Create()
    {
        return new CardPaymentGateway();
    }
}

public class CardPaymentGatewayFactoryProvider : AbstractPaymentGatewayFactoryProvider
{
    public override IPaymentGatewayFactory GetFactory()
    {
        return new CardPaymentGatewayFactory();
    }
}
