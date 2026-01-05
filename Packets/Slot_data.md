# Minecraft Java Edition Protocol — Slot Data

The **Slot data** format is how the Minecraft Java Edition protocol encodes an *item stack* inside inventory‑related packets. This includes any stack shown in GUIs such as inventories, chests, furnaces, etc., and is used by packets like:

- `Set Container Slot`
- `Set Container Content`
- `Click Container`
- Cursor slot representation

See inventory indexing for how slots are referenced. :contentReference[oaicite:1]{index=1}

---

## 📦 Slot Data — New Structured Format

Modern versions use **structured item components** instead of raw NBT. The general format encoded in the packet is:

| Field | Type | Meaning |
|-------|------|---------|
| **Item Count** | VarInt | Number of items in the stack |
| **Item ID** | Optional VarInt | The item’s numeric ID (distinct from block IDs) |
| **# Components to Add** | Optional VarInt | Count of structured components present |
| **# Components to Remove** | Optional VarInt | Count of component types to explicitly remove |
| **Components to Add** | Array (VarInt Enum + data) | Structured components to *add* |
| **Component Data** | Varies per component | The component’s contained structured data |
| **Components to Remove** | Array (VarInt Enum) | Components to *remove* (default values) |

> If **Item Count is zero**, no further fields follow — this represents an *empty slot*. :contentReference[oaicite:2]{index=2}

---

## 🧩 Component‑Driven System

Instead of unstructured NBT, item data is composed of **components**. Each component modifies or describes a particular aspect of the item (e.g., damage, custom name, enchantments). Examples include:

| Component ID | Name | Description |
|--------------|------|-------------|
| `minecraft:custom_data` | Component 0 | Arbitrary NBT‑style data |
| `minecraft:max_stack_size` | Component 1 | Overrides default max stack size |
| `minecraft:damage` | Component 3 | Current damage on the item |
| `minecraft:custom_name` | Component 5 | Item’s display name |
| `minecraft:enchantments` | Component 10 | The enchantments on the item |
| `minecraft:can_place_on` | Component 11 | Block placement restrictions |
| `minecraft:attribute_modifiers` | Component 13 | Attribute buffs/debuffs |
| `minecraft:food` | Component 20 | Hunger/saturation stats |
| `minecraft:use_remainder` | Component 22 | Remainder item after use |
| `minecraft:potion_contents` | Component 42 | Potion type/color metadata |
| `minecraft:bundle_contents` | Component 41 | Contents of a bundle item |

> Many additional data components exist. Each has its own structured subfields as defined by the protocol spec. :contentReference[oaicite:3]{index=3}

---

## 🧠 Structured Component Details

Each component carries **typed data** depending on its definition. For example:

- **minecraft:custom_name** contains a *Text Component* — the localized display name.
- **minecraft:enchantments** contains an array of `(enchantment_type, level)` pairs.
- **minecraft:can_place_on** holds a list of block predicates defining placement rules.  
These component definitions are part of the structured components section of the Slot Data spec. :contentReference[oaicite:4]{index=4}

---

## 🔐 Hashed Slot Format

In some packets (notably `Click Container`), a **hashed version** of the slot format is used:

| Field | Type | Meaning |
|-------|------|---------|
| **Has Item** | Boolean | If true, an item is present |
| **Item ID** | Optional VarInt | As before |
| **Item Count** | Optional VarInt | The count |
| **Components to Add** | Prefixed Array of Component IDs | Types of added components |
| **Component Data Hash** | Int | CRC32 hash of the actual component data |
| **Components to Remove** | Prefixed Array | Types of removed components |

In this *hashed* form, the actual values of the components are not sent — only their CRC32 hash — and this format is not fully documented for all cases. :contentReference[oaicite:5]{index=5}

---

## 🧪 Usage Patterns in Inventory Packets

### Empty Slot

If `Item Count == 0`:
- The slot is empty.
- No ID or components follow.

### Normal Stack

1. Read `Item Count` (non‑zero).
2. Read `Item ID`.
3. Read count of components to add (if present).
4. For each added component:
   - Parse the component type (Enum)
   - Parse its associated structured fields
5. Read count of components to remove (if present)
6. For each component to remove:
   - Parse the removed component type

The resulting stack comprises the base item type with *component overrides and additions*. :contentReference[oaicite:6]{index=6}

---

## 🧠 Component Semantics

- **Add vs Remove Lists:**  
  Minecraft uses *remove lists* to explicitly strip components that a base item would normally have (e.g., default enchantments).  
- **Order Matters:**  
  Clients expect structural order as defined by the registry of data component types.  
- **Empty Defaults:**  
  Absence of a component means the default behavior for that property (e.g., no custom name or attributes). :contentReference[oaicite:7]{index=7}

---

## 🛠 Implementation Notes

- When writing a serializer/deserializer:
  - Use VarInt for counts and ID fields.
  - Only include the components arrays if components are present.
  - For read/write of structured component data, follow each component’s specific subformat.  
- CRC hashing in the hashed slot variant is not fully documented — in practice, many implementations send a placeholder or compute CRC32 over actual structured data. :contentReference[oaicite:8]{index=8}

---

## 📌 Summary

- Slot Data represents items in inventory packets with *structured components* rather than raw NBT. :contentReference[oaicite:9]{index=9}  
- It consists of an item count, optional ID, and arrays of component data to add/remove. :contentReference[oaicite:10]{index=10}  
- Some inventory operations use a *hashed variant* where component values are replaced with CRC32 hashes. :contentReference[oaicite:11]{index=11}  

