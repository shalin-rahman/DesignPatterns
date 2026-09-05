interface IWorker
{
    void DoWork();
}

class Worker : IWorker
{
    void DoWork()
    {
        this.Persist();
    }

    void Persist()
    {
    }
}
