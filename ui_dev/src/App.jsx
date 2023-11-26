import { useEffect, useState } from 'react'
import './App.css'

function App() {
    const [count, setCount] = useState(0)
    let filename = "";
    useEffect(() => {
        console.log("Update to filename")
    }, [filename])
    filename = "1";
    return (
    <div> 
        <img src='players/a1.png'/>
    </div>
    )
}

export default App
