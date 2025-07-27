# Components Documentation

## AppBlockModal

A modal component that displays when an app is blocked due to distraction settings.

### Usage

```tsx
import AppBlockModal from "../components/AppBlockModal"
import { useAppBlockModal } from "../hooks/useAppBlockModal"

function MyComponent() {
  const { 
    isVisible, 
    showModal, 
    handleCloseApp, 
    handleUseFor5Mins 
  } = useAppBlockModal()

  return (
    <div>
      {/* Your app content */}
      
      {/* App Block Modal */}
      <AppBlockModal
        isVisible={isVisible}
        onClose={handleCloseApp}
        onUseFor5Mins={handleUseFor5Mins}
      />
      
      {/* Test button */}
      <button onClick={showModal}>
        Show Block Modal
      </button>
    </div>
  )
}
```

### Props

- `isVisible: boolean` - Controls whether the modal is shown
- `onClose: () => void` - Callback when "Close App" is clicked
- `onUseFor5Mins: () => void` - Callback when "Use for 5 mins" is clicked
- `onShowAgain?: () => void` - Optional callback for showing the modal again

### Features

- Responsive design that works on mobile and desktop
- Matches the Synapse app's design system with green accent colors
- Uses the "Wixmadefor text" font family
- Includes a backdrop overlay with blur effect
- Smooth animations and transitions
