#include <unordered_map>
#include <unordered_set>
#include <functional>
#include <chrono>
#include <variant>
#include <tuple>
// Empty-or-one-shared-value enums use the null pointer as their empty tag.
// This is a layout optimization for any enum of that shape, including Option.
template<class E,class S> std::size_t rusk_enum_index(const S& value){
    if constexpr(E::nullable) return value ? E::present : E::absent;
    else return value.index();
}
template<class S> struct RuskEnumPayloadView { S& field_0; };
template<class E,std::size_t I,class S> decltype(auto) rusk_enum_payload(S& value){
    if constexpr(E::nullable){
        static_assert(I==E::present);
        return RuskEnumPayloadView<S>{value};
    }else return (std::get<I>(value));
}
template<class E,std::size_t I,class... A> Obj<E> rusk_enum_make(A&&... args){
    if constexpr(E::nullable){
        if constexpr(I==E::present){
            static_assert(sizeof...(A)==1);
            return make_obj<E>(E{std::forward<A>(args)...});
        }else{
            static_assert(I==E::absent && sizeof...(A)==0);
            return make_obj<E>(E{});
        }
    }else{
        using V=std::variant_alternative_t<I,decltype(E::value)>;
        return make_obj<E>(E{decltype(E::value){std::in_place_index<I>,V{std::forward<A>(args)...}}});
    }
}
inline void rusk_hash_combine(std::size_t& hash,std::size_t part) {hash^=part+std::size_t(0x9e3779b9)+(hash<<6)+(hash>>2);}
inline std::uint64_t rusk_mix_hash(std::uint64_t x) {
    x += UINT64_C(0x9e3779b97f4a7c15);
    x = (x ^ (x >> 30)) * UINT64_C(0xbf58476d1ce4e5b9);
    x = (x ^ (x >> 27)) * UINT64_C(0x94d049bb133111eb);
    return x ^ (x >> 31);
}
inline std::uint64_t rusk_hash_seed() {
    static const auto seed = static_cast<std::uint64_t>(std::chrono::steady_clock::now().time_since_epoch().count());
    return seed;
}
// Calls through hash/equality policies may run arbitrary Rusk code. Keep a
// storage operation indivisible with respect to reentry into that storage.
inline thread_local Loc rusk_hash_location{"<hash>",0,0};
struct RuskHashLocation {
    Loc previous;
    explicit RuskHashLocation(Loc loc):previous(rusk_hash_location){rusk_hash_location=loc;}
    ~RuskHashLocation(){rusk_hash_location=previous;}
};
template<class Storage> struct RuskHashEntries : Storage {
    using Storage::Storage;
    mutable bool active=false;
    struct Guard {
        const RuskHashEntries& owner;
        explicit Guard(const RuskHashEntries& value):owner(value){owner.check_read(rusk_hash_location);owner.active=true;}
        ~Guard(){owner.active=false;}
    };
    void check_read(Loc loc) const {if(active)fail(loc,"container reentered during hash operation");}
    static Storage copy_storage(const RuskHashEntries& source){Guard guard(source);return Storage(static_cast<const Storage&>(source));}
    static Storage move_storage(RuskHashEntries& source){Guard guard(source);return Storage(std::move(static_cast<Storage&>(source)));}
    RuskHashEntries()=default;
    RuskHashEntries(const RuskHashEntries& source):Storage(copy_storage(source)){}
    RuskHashEntries(RuskHashEntries&& source):Storage(move_storage(source)){}
    RuskHashEntries& operator=(const RuskHashEntries& source){if(this!=&source){Guard a(*this);Guard b(source);Storage::operator=(static_cast<const Storage&>(source));}return *this;}
    RuskHashEntries& operator=(RuskHashEntries&& source){if(this!=&source){Guard a(*this);Guard b(source);Storage::operator=(std::move(static_cast<Storage&>(source)));}return *this;}
    template<class Key> auto find(const Key& key){Guard guard(*this);return Storage::find(key);}
    template<class Key> auto find(const Key& key) const {Guard guard(*this);return Storage::find(key);}
    template<class... A> auto emplace(A&&... args){Guard guard(*this);return Storage::emplace(std::forward<A>(args)...);}
    template<class... A> auto insert(A&&... args){Guard guard(*this);return Storage::insert(std::forward<A>(args)...);}
    template<class... A> auto erase(A&&... args){Guard guard(*this);return Storage::erase(std::forward<A>(args)...);}
    template<class Key> decltype(auto) operator[](Key&& key){Guard guard(*this);return Storage::operator[](std::forward<Key>(key));}
    void clear(){Guard guard(*this);Storage::clear();}
    void reserve(std::size_t count){Guard guard(*this);Storage::reserve(count);}
    void rehash(std::size_t count){Guard guard(*this);Storage::rehash(count);}
    void swap(RuskHashEntries& other){if(this==&other)return;Guard a(*this);Guard b(other);Storage::swap(other);}
    auto begin(){check_read(rusk_hash_location);return Storage::begin();}
    auto end(){check_read(rusk_hash_location);return Storage::end();}
    auto begin() const {check_read(rusk_hash_location);return Storage::begin();}
    auto end() const {check_read(rusk_hash_location);return Storage::end();}
    auto cbegin() const {check_read(rusk_hash_location);return Storage::cbegin();}
    auto cend() const {check_read(rusk_hash_location);return Storage::cend();}
};
template<class T,auto Hash> struct RuskHash {
    std::size_t operator()(const Obj<T>& x) const {
        return static_cast<std::size_t>(rusk_mix_hash(static_cast<std::uint64_t>(Hash(x,rusk_hash_location)) + rusk_hash_seed()));
    }
};
template<class T,auto Equal> struct RuskEqual {bool operator()(const Obj<T>& a,const Obj<T>& b) const{return Equal(a,b,rusk_hash_location);}};
template<class K,class V,auto Hash,auto Equal> struct RuskDict {
    using Entries=RuskHashEntries<std::unordered_map<Obj<K>,Obj<V>,RuskHash<K,Hash>,RuskEqual<K,Equal>>>;
    Entries entries;
    std::uint64_t version=0;
};
template<class K,auto Hash,auto Equal> struct RuskSet {
    using Entries=RuskHashEntries<std::unordered_set<Obj<K>,RuskHash<K,Hash>,RuskEqual<K,Equal>>>;
    Entries entries;
    std::uint64_t version=0;
};
template<class Owner> struct RuskHashIterator {
    Obj<Owner> owner;
    typename Owner::Entries::iterator position,end;
    std::uint64_t version;
    bool finished=false;
    bool advancing=false;
    explicit RuskHashIterator(Obj<Owner> value):owner(value),position(value->entries.begin()),end(value->entries.end()),version(value->version){}
};
template<class Owner> void rusk_guard_iter(const Obj<RuskHashIterator<Owner>>& it,Loc loc){
    if(!it->finished)it->owner->entries.check_read(loc);
    if(!it->finished && it->version!=it->owner->version)fail(loc,"container changed during iteration");
}

struct RuskHashAdvanceGuard {
    bool& active;
    RuskHashAdvanceGuard(bool& value,Loc loc):active(value){if(active)fail(loc,"iterator reentered during key copy");active=true;}
    ~RuskHashAdvanceGuard(){active=false;}
};
