namespace Demo
{
    public class Order
    {
        public void Ship()
        {
            this.Validate();
            warehouse.Reserve();
            a.b.c.Foo();
            var x = a.b.c.Field;
        }
    }
}
