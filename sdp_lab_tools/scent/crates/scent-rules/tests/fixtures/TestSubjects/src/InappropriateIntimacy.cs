class A
{
    private B b;
    void CallB0() { b.Foo(); }
    void CallB1() { b.Foo(); }
    void CallB2() { b.Foo(); }
    void CallB3() { b.Foo(); }
    void CallB4() { b.Foo(); }
    void CallB5() { b.Foo(); }
}

class B
{
    private A a;
    void Foo() { a.Bar(); }
    void Bar() {}
}
