namespace Demo.Models
{
    public class Order
    {
        private int total;

        public string Status { get; set; }

        public void Ship()
        {
            this.Validate();
        }

        // scent:disable LONG_METHOD
        public void Validate()
        {
        }
        // scent:enable LONG_METHOD
    }

    public class OrderFactory
    {
        public Order Create()
        {
            return new Order();
        }
    }
}
