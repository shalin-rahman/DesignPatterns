class Order
{
    private Warehouse warehouse;

    void Ship()
    {
        warehouse.Reserve();
        warehouse.Pack();
        warehouse.Dispatch();
        warehouse.Notify();
    }
}

class Warehouse
{
    void Reserve()
    {
    }

    void Pack()
    {
    }

    void Dispatch()
    {
    }

    void Notify()
    {
    }
}
