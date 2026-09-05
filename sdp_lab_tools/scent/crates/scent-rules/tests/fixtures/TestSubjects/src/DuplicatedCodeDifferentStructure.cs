class Order
{
    void A()
    {
        int total = 1;
        total = total + 1;
        if (total > 0) { total = 0; }
        return;
    }

    void B()
    {
        int amount = 1;
        amount = amount - 1;
        while (amount > 0) { amount = 0; }
        return;
    }
}
