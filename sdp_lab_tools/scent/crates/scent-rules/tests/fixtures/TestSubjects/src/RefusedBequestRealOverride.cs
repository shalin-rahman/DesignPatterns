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
        this.Persist();
    }

    void Persist()
    {
    }
}
