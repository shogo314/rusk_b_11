#include <map>
#include <set>
// Guard the native tree while comparisons or cleanup can call Rusk code.
template<class Storage> struct RuskOrderedEntries : Storage {
    using Storage::Storage;
    mutable bool active=false;
    struct Guard {
        const RuskOrderedEntries& owner;
        explicit Guard(const RuskOrderedEntries& value):owner(value){owner.check_read(rusk_hash_location);owner.active=true;}
        ~Guard(){owner.active=false;}
    };
    void check_read(Loc loc) const {if(active)fail(loc,"container reentered during ordered operation");}
    static Storage copy_storage(const RuskOrderedEntries& source){Guard guard(source);return Storage(static_cast<const Storage&>(source));}
    static Storage move_storage(RuskOrderedEntries& source){Guard guard(source);return Storage(std::move(static_cast<Storage&>(source)));}
    RuskOrderedEntries()=default;
    RuskOrderedEntries(const RuskOrderedEntries& source):Storage(copy_storage(source)){}
    RuskOrderedEntries(RuskOrderedEntries&& source):Storage(move_storage(source)){}
    RuskOrderedEntries& operator=(const RuskOrderedEntries& source){if(this!=&source){Guard a(*this);Guard b(source);Storage::operator=(static_cast<const Storage&>(source));}return *this;}
    RuskOrderedEntries& operator=(RuskOrderedEntries&& source){if(this!=&source){Guard a(*this);Guard b(source);Storage::operator=(std::move(static_cast<Storage&>(source)));}return *this;}
    template<class Key> auto lower_bound(const Key& key){Guard guard(*this);return Storage::lower_bound(key);}
    template<class Key> auto upper_bound(const Key& key){Guard guard(*this);return Storage::upper_bound(key);}
    template<class Key> auto count(const Key& key)const{Guard guard(*this);return Storage::count(key);}
    template<class Key> auto find(const Key& key){Guard guard(*this);return Storage::find(key);}
    template<class Key> auto find(const Key& key) const {Guard guard(*this);return Storage::find(key);}
    template<class... A> auto emplace(A&&... args){Guard guard(*this);return Storage::emplace(std::forward<A>(args)...);}
    template<class... A> auto insert(A&&... args){Guard guard(*this);return Storage::insert(std::forward<A>(args)...);}
    template<class... A> auto erase(A&&... args){Guard guard(*this);return Storage::erase(std::forward<A>(args)...);}
    template<class Key> decltype(auto) operator[](Key&& key){Guard guard(*this);return Storage::operator[](std::forward<Key>(key));}
    void clear(){Guard guard(*this);Storage::clear();}
    void reserve(std::size_t count){Guard guard(*this);Storage::reserve(count);}
    void rehash(std::size_t count){Guard guard(*this);Storage::rehash(count);}
    void swap(RuskOrderedEntries& other){if(this==&other)return;Guard a(*this);Guard b(other);Storage::swap(other);}
    auto begin(){check_read(rusk_hash_location);return Storage::begin();}
    auto end(){check_read(rusk_hash_location);return Storage::end();}
    auto begin() const {check_read(rusk_hash_location);return Storage::begin();}
    auto end() const {check_read(rusk_hash_location);return Storage::end();}
    auto cbegin() const {check_read(rusk_hash_location);return Storage::cbegin();}
    auto cend() const {check_read(rusk_hash_location);return Storage::cend();}
};
template<class K,auto Less> struct RuskOrderedLess {bool operator()(const Obj<K>& a,const Obj<K>& b)const{return Less(a,b,rusk_hash_location);}};
template<class K,auto Less> struct RuskOrderedSet {
 using Entries=RuskOrderedEntries<std::set<Obj<K>,RuskOrderedLess<K,Less>>>;
 Entries entries;std::uint64_t version=0;
};
template<class K,auto Less> struct RuskMultiSet {
 using Entries=RuskOrderedEntries<std::multiset<Obj<K>,RuskOrderedLess<K,Less>>>;
 Entries entries;std::uint64_t version=0;
};
template<class K,class V,auto Less> struct RuskOrderedDict {
 using Entries=RuskOrderedEntries<std::map<Obj<K>,Obj<V>,RuskOrderedLess<K,Less>>>;
 Entries entries;std::uint64_t version=0;
};
