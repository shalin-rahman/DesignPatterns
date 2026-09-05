interface IWorker
{
    void DoWork();
    void DoOther();
}

class Worker : IWorker
{
    void DoWork()
    {
        this.Persist();
    }

    void DoOther()
    {
        throw new NotImplementedException();
    }

    void Persist()
    {
    }
}
