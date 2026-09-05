namespace Demo
{
    public class Order
    {
        private Warehouse warehouse;

        public void Ship()
        {
            Depot warehouse = new Depot();
            warehouse.Reserve();
        }
    }

    public class Warehouse
    {
        public void Reserve()
        {
        }
    }

    public class Depot
    {
        public void Reserve()
        {
        }
    }
}
