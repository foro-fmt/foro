#include <iostream>
#include <string>
#include <vector>
#include <algorithm>
#include <map>
#include <cmath>
#include <cstdlib>

class Foo {
  public:
    Foo() : x_(0) {}
    Foo(int x) : x_(x) {}
    int getX() const { return x_; }
    void setX(int x) { x_ = x; }

  private:
    int x_;
};

class Bar {
  public:
    Bar() : y_(0.0) {}
    Bar(double y) : y_(y) {}
    double getY() const { return y_; }
    void setY(double y) { y_ = y; }

  private:
    double y_;
};

int add(int a, int b) { return a + b; }

double mul(double a, double b) { return a * b; }

std::string greet(const std::string &name) { return "Hello, " + name + "!"; }

int trivialFunction1(int x) { return x + 1; }
int trivialFunction2(int x) { return x + 2; }
int trivialFunction3(int x) { return x + 3; }
int trivialFunction4(int x) { return x + 4; }
int trivialFunction5(int x) { return x + 5; }
int trivialFunction6(int x) { return x + 6; }
int trivialFunction7(int x) { return x + 7; }
int trivialFunction8(int x) { return x + 8; }
int trivialFunction9(int x) { return x + 9; }
int trivialFunction10(int x) { return x + 10; }

int trivialFunction11(int x) { return x + 11; }
int trivialFunction12(int x) { return x + 12; }
int trivialFunction13(int x) { return x + 13; }
int trivialFunction14(int x) { return x + 14; }
int trivialFunction15(int x) { return x + 15; }
int trivialFunction16(int x) { return x + 16; }
int trivialFunction17(int x) { return x + 17; }
int trivialFunction18(int x) { return x + 18; }
int trivialFunction19(int x) { return x + 19; }
int trivialFunction20(int x) { return x + 20; }

int trivialFunction21(int x) { return x + 21; }
int trivialFunction22(int x) { return x + 22; }
int trivialFunction23(int x) { return x + 23; }
int trivialFunction24(int x) { return x + 24; }
int trivialFunction25(int x) { return x + 25; }
int trivialFunction26(int x) { return x + 26; }
int trivialFunction27(int x) { return x + 27; }
int trivialFunction28(int x) { return x + 28; }
int trivialFunction29(int x) { return x + 29; }
int trivialFunction30(int x) { return x + 30; }

int trivialFunction31(int x) { return x + 31; }
int trivialFunction32(int x) { return x + 32; }
int trivialFunction33(int x) { return x + 33; }
int trivialFunction34(int x) { return x + 34; }
int trivialFunction35(int x) { return x + 35; }
int trivialFunction36(int x) { return x + 36; }
int trivialFunction37(int x) { return x + 37; }
int trivialFunction38(int x) { return x + 38; }
int trivialFunction39(int x) { return x + 39; }
int trivialFunction40(int x) { return x + 40; }

int trivialFunction41(int x) { return x + 41; }
int trivialFunction42(int x) { return x + 42; }
int trivialFunction43(int x) { return x + 43; }
int trivialFunction44(int x) { return x + 44; }
int trivialFunction45(int x) { return x + 45; }
int trivialFunction46(int x) { return x + 46; }
int trivialFunction47(int x) { return x + 47; }
int trivialFunction48(int x) { return x + 48; }
int trivialFunction49(int x) { return x + 49; }
int trivialFunction50(int x) { return x + 50; }

int trivialFunction51(int x) { return x + 51; }
int trivialFunction52(int x) { return x + 52; }
int trivialFunction53(int x) { return x + 53; }
int trivialFunction54(int x) { return x + 54; }
int trivialFunction55(int x) { return x + 55; }
int trivialFunction56(int x) { return x + 56; }
int trivialFunction57(int x) { return x + 57; }
int trivialFunction58(int x) { return x + 58; }
int trivialFunction59(int x) { return x + 59; }
int trivialFunction60(int x) { return x + 60; }

int trivialFunction61(int x) { return x + 61; }
int trivialFunction62(int x) { return x + 62; }
int trivialFunction63(int x) { return x + 63; }
int trivialFunction64(int x) { return x + 64; }
int trivialFunction65(int x) { return x + 65; }
int trivialFunction66(int x) { return x + 66; }
int trivialFunction67(int x) { return x + 67; }
int trivialFunction68(int x) { return x + 68; }
int trivialFunction69(int x) { return x + 69; }
int trivialFunction70(int x) { return x + 70; }

int trivialFunction71(int x) { return x + 71; }
int trivialFunction72(int x) { return x + 72; }
int trivialFunction73(int x) { return x + 73; }
int trivialFunction74(int x) { return x + 74; }
int trivialFunction75(int x) { return x + 75; }
int trivialFunction76(int x) { return x + 76; }
int trivialFunction77(int x) { return x + 77; }
int trivialFunction78(int x) { return x + 78; }
int trivialFunction79(int x) { return x + 79; }
int trivialFunction80(int x) { return x + 80; }

int trivialFunction81(int x) { return x + 81; }
int trivialFunction82(int x) { return x + 82; }
int trivialFunction83(int x) { return x + 83; }
int trivialFunction84(int x) { return x + 84; }
int trivialFunction85(int x) { return x + 85; }
int trivialFunction86(int x) { return x + 86; }
int trivialFunction87(int x) { return x + 87; }
int trivialFunction88(int x) { return x + 88; }
int trivialFunction89(int x) { return x + 89; }
int trivialFunction90(int x) { return x + 90; }

int trivialFunction91(int x) { return x + 91; }
int trivialFunction92(int x) { return x + 92; }
int trivialFunction93(int x) { return x + 93; }
int trivialFunction94(int x) { return x + 94; }
int trivialFunction95(int x) { return x + 95; }
int trivialFunction96(int x) { return x + 96; }
int trivialFunction97(int x) { return x + 97; }
int trivialFunction98(int x) { return x + 98; }
int trivialFunction99(int x) { return x + 99; }
int trivialFunction100(int x) { return x + 100; }

class Baz {
  public:
    Baz() : str_("default") {}
    Baz(const std::string &s) : str_(s) {}
    std::string getStr() const { return str_; }
    void setStr(const std::string &s) { str_ = s; }

  private:
    std::string str_;
};

class Qux {
  public:
    Qux() : values_({1, 2, 3}) {}
    void addValue(int v) { values_.push_back(v); }
    int sum() const {
        int s = 0;
        for (auto v : values_)
            s += v;
        return s;
    }

  private:
    std::vector<int> values_;
};

class Complex {
  public:
    Complex(double r, double i) : real_(r), imag_(i) {}
    double real() const { return real_; }
    double imag() const { return imag_; }
    Complex operator+(const Complex &other) const {
        return Complex(real_ + other.real_, imag_ + other.imag_);
    }

  private:
    double real_;
    double imag_;
};

class DummyA {
  public:
    DummyA() : a_(0) {}
    int inc() { return ++a_; }

  private:
    int a_;
};

class DummyB {
  public:
    DummyB() {
        for (int i = 0; i < 50; ++i)
            data_.push_back(i);
    }
    int get(int idx) const {
        if (idx >= 0 && idx < (int)data_.size())
            return data_[idx];
        return -1;
    }

  private:
    std::vector<int> data_;
};

class DummyC {
  public:
    DummyC() : m_("none") {}
    void setMsg(const std::string &msg) { m_ = msg; }
    std::string getMsg() const { return m_; }

  private:
    std::string m_;
};

int dummyFuncA(int x) { return x * x; }
int dummyFuncB(int x) { return x - 10; }
int dummyFuncC(int x, int y) { return x * y + 2; }

int anotherFunc1(int x) { return x + 101; }
int anotherFunc2(int x) { return x + 102; }
int anotherFunc3(int x) { return x + 103; }
int anotherFunc4(int x) { return x + 104; }
int anotherFunc5(int x) { return x + 105; }
int anotherFunc6(int x) { return x + 106; }
int anotherFunc7(int x) { return x + 107; }
int anotherFunc8(int x) { return x + 108; }
int anotherFunc9(int x) { return x + 109; }
int anotherFunc10(int x) { return x + 110; }

class DummyD {
  public:
    DummyD() : val_(0) {}
    void add(int v) { val_ += v; }
    int val() const { return val_; }

  private:
    int val_;
};

class DummyE {
  public:
    DummyE() : dmap_({{"key1", 1}, {"key2", 2}}) {}
    int getVal(const std::string &k) const {
        auto it = dmap_.find(k);
        if (it != dmap_.end())
            return it->second;
        return -1;
    }

  private:
    std::map<std::string, int> dmap_;
};

class DummyF {
  public:
    DummyF() {}
    bool isEven(int x) const { return x % 2 == 0; }
};

int main() {
    Foo foo(42);
    Bar bar(3.14);
    Baz baz("hello");
    Qux qux;
    Complex c1(1.0, 2.0);
    Complex c2(3.0, 4.0);
    Complex c3 = c1 + c2;

    std::cout << greet("World") << std::endl;
    std::cout << "foo: " << foo.getX() << std::endl;
    std::cout << "bar: " << bar.getY() << std::endl;
    std::cout << "baz: " << baz.getStr() << std::endl;
    std::cout << "qux sum: " << qux.sum() << std::endl;
    std::cout << "c3: (" << c3.real() << ", " << c3.imag() << ")" << std::endl;

    int val = 0;
    val = trivialFunction1(val);
    val = trivialFunction2(val);
    val = trivialFunction3(val);
    val = trivialFunction4(val);
    val = trivialFunction5(val);
    val = trivialFunction6(val);
    val = trivialFunction7(val);
    val = trivialFunction8(val);
    val = trivialFunction9(val);
    val = trivialFunction10(val);
    val = trivialFunction11(val);
    val = trivialFunction12(val);
    val = trivialFunction13(val);
    val = trivialFunction14(val);
    val = trivialFunction15(val);
    val = trivialFunction16(val);
    val = trivialFunction17(val);
    val = trivialFunction18(val);
    val = trivialFunction19(val);
    val = trivialFunction20(val);
    val = trivialFunction21(val);
    val = trivialFunction22(val);
    val = trivialFunction23(val);
    val = trivialFunction24(val);
    val = trivialFunction25(val);
    val = trivialFunction26(val);
    val = trivialFunction27(val);
    val = trivialFunction28(val);
    val = trivialFunction29(val);
    val = trivialFunction30(val);
    val = trivialFunction31(val);
    val = trivialFunction32(val);
    val = trivialFunction33(val);
    val = trivialFunction34(val);
    val = trivialFunction35(val);
    val = trivialFunction36(val);
    val = trivialFunction37(val);
    val = trivialFunction38(val);
    val = trivialFunction39(val);
    val = trivialFunction40(val);
    val = trivialFunction41(val);
    val = trivialFunction42(val);
    val = trivialFunction43(val);
    val = trivialFunction44(val);
    val = trivialFunction45(val);
    val = trivialFunction46(val);
    val = trivialFunction47(val);
    val = trivialFunction48(val);
    val = trivialFunction49(val);
    val = trivialFunction50(val);
    val = trivialFunction51(val);
    val = trivialFunction52(val);
    val = trivialFunction53(val);
    val = trivialFunction54(val);
    val = trivialFunction55(val);
    val = trivialFunction56(val);
    val = trivialFunction57(val);
    val = trivialFunction58(val);
    val = trivialFunction59(val);
    val = trivialFunction60(val);
    val = trivialFunction61(val);
    val = trivialFunction62(val);
    val = trivialFunction63(val);
    val = trivialFunction64(val);
    val = trivialFunction65(val);
    val = trivialFunction66(val);
    val = trivialFunction67(val);
    val = trivialFunction68(val);
    val = trivialFunction69(val);
    val = trivialFunction70(val);
    val = trivialFunction71(val);
    val = trivialFunction72(val);
    val = trivialFunction73(val);
    val = trivialFunction74(val);
    val = trivialFunction75(val);
    val = trivialFunction76(val);
    val = trivialFunction77(val);
    val = trivialFunction78(val);
    val = trivialFunction79(val);
    val = trivialFunction80(val);
    val = trivialFunction81(val);
    val = trivialFunction82(val);
    val = trivialFunction83(val);
    val = trivialFunction84(val);
    val = trivialFunction85(val);
    val = trivialFunction86(val);
    val = trivialFunction87(val);
    val = trivialFunction88(val);
    val = trivialFunction89(val);
    val = trivialFunction90(val);
    val = trivialFunction91(val);
    val = trivialFunction92(val);
    val = trivialFunction93(val);
    val = trivialFunction94(val);
    val = trivialFunction95(val);
    val = trivialFunction96(val);
    val = trivialFunction97(val);
    val = trivialFunction98(val);
    val = trivialFunction99(val);
    val = trivialFunction100(val);

    std::cout << "Final val: " << val << std::endl;

    DummyA da;
    std::cout << "DummyA inc: " << da.inc() << std::endl;

    DummyB db;
    std::cout << "DummyB get(10): " << db.get(10) << std::endl;

    DummyC dc;
    dc.setMsg("test");
    std::cout << "DummyC msg: " << dc.getMsg() << std::endl;

    std::cout << "dummyFuncA(10): " << dummyFuncA(10) << std::endl;
    std::cout << "dummyFuncB(20): " << dummyFuncB(20) << std::endl;
    std::cout << "dummyFuncC(3,4): " << dummyFuncC(3, 4) << std::endl;

    DummyD dd;
    dd.add(100);
    std::cout << "DummyD val: " << dd.val() << std::endl;

    DummyE de;
    std::cout << "DummyE getVal(\"key1\"): " << de.getVal("key1") << std::endl;
    std::cout << "DummyE getVal(\"unknown\"): " << de.getVal("unknown")
              << std::endl;

    DummyF df;
    std::cout << "DummyF isEven(42): " << df.isEven(42) << std::endl;

    std::cout << "anotherFunc1(0): " << anotherFunc1(0) << std::endl;
    std::cout << "anotherFunc10(0): " << anotherFunc10(0) << std::endl;

    return 0;
}
