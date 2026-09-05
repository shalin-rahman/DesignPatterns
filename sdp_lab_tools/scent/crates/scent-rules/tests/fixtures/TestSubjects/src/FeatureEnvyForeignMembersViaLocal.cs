class Order
{
    void Ship()
    {
        Warehouse warehouse = new Warehouse();
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
