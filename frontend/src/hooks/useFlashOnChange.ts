import { useEffect, useState } from 'react';

function useFlashOnChange<T>(value: T, ms = 400) {
  const [isFlashing, setIsFlashing] = useState(false);
  
  useEffect(() => {
    setIsFlashing(true);
    const timer = setTimeout(() => setIsFlashing(false), ms);
    return () => clearTimeout(timer);
  }, [value, ms]);
  
  return isFlashing;
}

export default useFlashOnChange;
