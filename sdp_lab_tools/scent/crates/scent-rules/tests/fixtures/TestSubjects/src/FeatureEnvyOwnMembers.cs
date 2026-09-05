class Order
{
    private Warehouse warehouse;

    void Ship()
    {
        this.Validate();
        this.Log();
        this.Persist();
        warehouse.Notify();
    }

    void Validate()
    {
    }

    void Log()
    {
    }

    void Persist()
    {
    }
}

class Warehouse
{
    void Notify()
    {
    }
}
