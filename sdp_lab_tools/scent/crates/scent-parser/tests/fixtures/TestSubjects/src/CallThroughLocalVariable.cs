namespace Demo
{
    public class Order
    {
        public void Ship()
        {
            Warehouse w = new Warehouse();
            w.Reserve();
        }
    }

    public class Warehouse
    {
        public void Reserve()
        {
        }
    }
}
