# CoPiReMap Plugin - CLAP Validator Test Plan

## Issue Description
The CoPiReMap plugin causes DAW crashes when:
1. Changing projects
2. Removing the plugin from the project

## Test Environment
- Plugin: CoPiReMap (CLAP version)
- Tool: clap-validator.exe
- Platform: Windows
- Build: Release

## Pre-Test Checklist
- [ ] Build plugin in release mode: `cargo build --package copiremap --release`
- [ ] Locate plugin DLL: `target/release/copiremap.clap`
- [ ] Ensure clap-validator.exe is available

## Test Procedures

### Test 1: Basic Plugin Loading
**Command:**
```powershell
clap-validator.exe validate target/release/copiremap.clap
```

**Expected Result:**
- Plugin loads successfully
- No errors during initialization
- Plugin metadata is correct

### Test 2: Plugin Instantiation and Cleanup
**Command:**
```powershell
clap-validator.exe validate target/release/copiremap.clap --test-all
```

**Focus Areas:**
- Check for proper initialization
- Monitor memory allocations
- **Critical: Verify cleanup sequence**
  - Check `Drop` implementations are called
  - Verify window resources are released
  - Check audio processing resources are freed

### Test 3: Multiple Load/Unload Cycles
**Purpose:** Simulate changing projects or removing/re-adding plugin

**Test Steps:**
1. Load plugin instance
2. Initialize audio processing
3. Deactivate plugin
4. Destroy plugin instance
5. Repeat 10 times

**What to Monitor:**
- Memory leaks
- Dangling pointers
- Resource cleanup
- Window handle cleanup
- Thread cleanup

### Test 4: GUI Operations
**Focus:**
- Open GUI
- Close GUI
- **Critical: Window cleanup on close**
  - Verify EditorHandle Drop is called
  - Check window adapter cleanup
  - Verify cyclic reference is broken

### Test 5: Audio Processing Under Load
**Test:**
- Start audio processing
- Stop audio processing
- **Critical: Check cleanup of:**
  - AudioProcess96 nodes (96 instances)
  - Filter states (lpf, hpf)
  - Delay buffers
  - Gate states
  - Pitch shifter resources

## Potential Issues Identified

### 1. Window/GUI Cleanup
**Location:** `nih_plug_slint/src/editor.rs` and `window_adapter.rs`

**Potential Issue:**
```rust
impl Drop for EditorHandle {
    fn drop(&mut self) {
        let window_adapter_ptr = self.window_adapter_ptr.swap(null_mut(), Ordering::Relaxed);
        unsafe { Rc::from_raw(window_adapter_ptr) };
    }
}
```

**Risk:** If Drop is not called properly, window resources may leak

### 2. Message Window Thread
**Location:** `plugin-canvas/src/platform/win32/window.rs`

**Potential Issue:**
```rust
let message_window = Arc::new(MessageWindow::new(hwnd).unwrap());
std::thread::spawn({
    let message_window = message_window.clone();
    move || message_window.run()
});
```

**Risk:** Background thread may not be properly terminated

### 3. Frame Pacing Thread
**Location:** `plugin-canvas/src/platform/win32/window.rs`

**Potential Issue:**
```rust
std::thread::spawn({
    let moved = moved.clone();
    move || frame_pacing_thread(hwnd, moved)
});
```

**Risk:** Background thread may access freed window handle

### 4. AudioProcess96 Vector (96 instances)
**Location:** `plugins/copiremap/src/lib.rs`

**Potential Issue:**
- 96 pitch shift nodes with complex state
- Each has filters, delays, and pitch shifter
- Reset is called but may not fully clean up resources

```rust
fn reset(&mut self) {
    for ap in self.audio_process96.iter_mut() {
        ap.reset();
    }
}
```

### 5. Arc/Atomic Reference Cycles
**Location:** Throughout parameter system

**Potential Issue:**
- Multiple Arc<AtomicBool> shared between params and plugin
- May create reference cycles preventing cleanup

## Recommended Fixes

### Fix 1: Add explicit deactivate method
```rust
impl Plugin for CoPiReMapPlugin {
    // ... existing code ...
    
    fn deactivate(&mut self) {
        // Ensure all audio processing is stopped
        for ap in self.audio_process96.iter_mut() {
            ap.reset();
        }
        // Clear any pending updates
        self.update_lowpass.store(false, Ordering::Release);
        self.update_highpass.store(false, Ordering::Release);
        // ... clear all other atomic flags
    }
}
```

### Fix 2: Implement Drop for CoPiReMapPlugin
```rust
impl Drop for CoPiReMapPlugin {
    fn drop(&mut self) {
        // Explicit cleanup
        for ap in self.audio_process96.iter_mut() {
            ap.reset();
        }
    }
}
```

### Fix 3: Better thread cleanup in Win32 window
- Add thread handle tracking
- Implement proper thread join on window destruction
- Send termination message to background threads

### Fix 4: Window resource cleanup verification
- Add logging to Drop implementations
- Verify RevokeDragDrop is called
- Verify DestroyWindow completes
- Verify UnhookWindowsHookEx succeeds

## Test Execution

### Step 1: Run Basic Validation
```powershell
cd C:\Users\LogicCuteGuy\Downloads\my_audio_plugin
cargo build --package copiremap --release
clap-validator.exe validate target\release\copiremap.clap
```

### Step 2: Run Full Test Suite
```powershell
clap-validator.exe validate target\release\copiremap.clap --test-all
```

### Step 3: Check for Common Issues
```powershell
# Run with sanitizers if available
$env:RUSTFLAGS="-Z sanitizer=address"
cargo build --package copiremap --target x86_64-pc-windows-msvc
```

## Expected Results

### Success Criteria:
- [ ] All clap-validator tests pass
- [ ] No memory leaks detected
- [ ] Proper cleanup sequence executed
- [ ] No crashes during cleanup
- [ ] GUI closes cleanly
- [ ] Audio processing stops cleanly

### Failure Indicators:
- [ ] Crashes during unload
- [ ] Memory not freed
- [ ] Window handles not released
- [ ] Threads not terminated
- [ ] Access violations

## Notes
- Test on clean Windows system
- Monitor Task Manager for lingering processes
- Check for orphaned window handles
- Verify DLL is fully unloaded after test

## Additional Testing
- Test in actual DAW (Reaper, Bitwig, FL Studio)
- Monitor with Process Explorer
- Use API Monitor to track Win32 API calls
- Verify cleanup with Handle.exe utility
