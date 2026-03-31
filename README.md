# FlakeRS – Flaker Bond System

FlakeRS is a decentralized accountability system built using Soroban smart contracts on the Stellar network.

It solves a simple problem: **people flaking on plans**.

Participants stake tokens before an event. Those who show up get rewarded. Those who don’t lose their stake.

---

## 📌 Contract Information

### Contract ID

```
CD5AHFKLNUCVSBKSAJHWV6WKJDRWWHNBV3AUIFOI3CLWSXWXSWS3R7OE
```

---

### 📸 Deployment Proof

---

## 🧠 How It Works

### 1. Initialize Event

- Host creates an event
- Defines:
  - Stake amount
  - Token address
  - Secret hash

---

### 2. Join Event

- Participants approve token spending
- Stake is transferred into the contract

---

### 3. Check-In

- Users submit a secret (e.g., QR code)
- Contract verifies using SHA-256 hash

---

### 4. Settle

- Host finalizes the event
- Total pool is distributed among attendees

---

## 💸 Example

| Participants | Stake | Total Pool | Attendees | Payout Each |
| ------------ | ----- | ---------- | --------- | ----------- |
| 3            | 100   | 300        | 2         | 150         |

---

## ⚙️ Contract Functions

**initialize**  
Creates an event with parameters

**join**  
Allows a participant to stake tokens and join

**check_in**  
Verifies attendance using a hashed secret

**settle**  
Distributes funds to attendees

---

## 🏗️ Tech Stack

- Rust (#![no_std])
- Soroban SDK
- Stellar Smart Contracts (WASM-based)

---

## 🚀 Build & Test

```bash
cargo build --target wasm32-unknown-unknown --release
cargo test
```

---

## 🔐 Security Notes

- Requires token approval before joining
- Prevents duplicate joins and check-ins
- Uses SHA-256 hashing for validation
- Funds are locked until settlement

⚠️ This project is experimental and not audited.

---

## 💡 Use Cases

- Friend meetups
- Study sessions
- Fitness accountability
- Event attendance incentives
- IRL coordination for DAOs

---

## 📁 Project Structure

```
.
├── src/
│   ├── lib.rs
│   └── test.rs
├── Cargo.toml
└── README.md
```

---

## 📎 Future Improvements

- Replace Vec with Map for efficiency
- Add time-based logic
- Auto-settlement
