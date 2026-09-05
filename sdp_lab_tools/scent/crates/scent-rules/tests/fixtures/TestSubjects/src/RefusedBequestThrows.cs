class Base
{
    void Save()
    {
    }
}

class Derived : Base
{
    void Save()
    {
        throw new NotImplementedException();
    }
}
