#include <deque>
#include <bit>
struct RuskIntArray {
    using value_type=std::int64_t;
    std::vector<std::int64_t> values;
    std::uint64_t version=0;
    std::size_t size()const{return values.size();}
    std::int64_t at(std::size_t i)const{return values[i];}
};
struct RuskBitVec {
    using value_type=bool;
    std::vector<std::uint64_t> words;
    std::size_t length=0;
    std::uint64_t version=0;
    RuskBitVec()=default;
    RuskBitVec(std::int64_t n,bool value,Loc loc){if(n<0)fail(loc,"negative bit length");length=static_cast<std::size_t>(n);words.assign(length/64+(length%64!=0),value?~std::uint64_t(0):0);trim();}
    std::size_t size()const{return length;}
    bool at(std::size_t i)const{return ((words[i/64]>>(i%64))&1)!=0;}
    void set(std::size_t i,bool value){auto mask=std::uint64_t(1)<<(i%64);if(value)words[i/64]|=mask;else words[i/64]&=~mask;}
    void trim(){if(!words.empty()&&length%64)words.back()&=(std::uint64_t(1)<<(length%64))-1;}
    std::size_t count()const{std::size_t result=0;for(auto w:words)result+=std::popcount(w);return result;}
    static RuskBitVec parse(const std::string&s,Loc loc){auto result=RuskBitVec(checked_int(s.size(),loc),false,loc);for(std::size_t i=0;i<s.size();++i){if(s[i]!='0'&&s[i]!='1')fail(loc,"invalid bit string");result.set(s.size()-1-i,s[i]=='1');}return result;}
    std::string str()const{std::string result(length,'0');for(std::size_t i=0;i<length;++i)if(at(i))result[length-1-i]='1';return result;}
    static RuskBitVec shift(const RuskBitVec& a,std::int64_t amount,bool left,Loc loc){auto n=amount<0?std::uint64_t(0)-static_cast<std::uint64_t>(amount):static_cast<std::uint64_t>(amount);if(amount<0)left=!left;RuskBitVec r(checked_int(a.length,loc),false,loc);if(n>=a.length)return r;auto whole=static_cast<std::size_t>(n/64);auto part=static_cast<unsigned>(n%64);for(std::size_t i=0;i<a.words.size();++i){if(left){if(i+whole<r.words.size())r.words[i+whole]|=a.words[i]<<part;if(part&&i+whole+1<r.words.size())r.words[i+whole+1]|=a.words[i]>>(64-part);}else{if(i>=whole)r.words[i-whole]|=a.words[i]>>part;if(part&&i>whole)r.words[i-whole-1]|=a.words[i]<<(64-part);}}r.trim();return r;}
};
template<std::int64_t N> struct RuskBits:RuskBitVec {
    RuskBits()=default;
    explicit RuskBits(RuskBitVec value):RuskBitVec(std::move(value)){}
};
template<class T> struct RuskDeque {
    std::deque<Obj<T>> values;
    std::uint64_t version=0;
    std::size_t size()const{return values.size();}
    Obj<T> at(std::size_t i)const{return values[i];}
};
template<class T,std::int64_t N> struct RuskArray {
    std::vector<Obj<T>> values;
    std::uint64_t version=0;
    std::size_t size()const{return values.size();}
    Obj<T> at(std::size_t i)const{return values[i];}
};
template<class Owner> struct RuskLinearIter {
    Obj<Owner> owner;
    std::size_t index=0;
    std::uint64_t version=0;
    bool finished=false;
    explicit RuskLinearIter(Obj<Owner> value):owner(std::move(value)),version(owner->version){}
};
template<class Owner> void rusk_guard_iter(const Obj<RuskLinearIter<Owner>>& it,Loc loc){if(!it->finished&&it->version!=it->owner->version)fail(loc,"container changed during iteration");}
