import React, {
    useState,
    useEffect,
    useContext,
    createContext,
    useMemo,
    useCallback
} from 'react'

// Define a type for user
type User = {
    id: number
    name: string
    email: string
}

// Define a type for theme
type Theme = 'light' | 'dark'

// Create a context for theme
const ThemeContext = createContext<{ theme: Theme; toggleTheme: () => void }>({
    theme: 'light',
    toggleTheme: () => {}
})

// Custom hook for using the ThemeContext
const useTheme = () => {
    return useContext(ThemeContext)
}

// Component to provide theme context
const ThemeProvider: React.FC<{ children: React.ReactNode }> = ({
    children
}) => {
    const [theme, setTheme] = useState<Theme>('light')

    const toggleTheme = useCallback(() => {
        setTheme((prev) => (prev === 'light' ? 'dark' : 'light'))
    }, [])

    const value = useMemo(() => ({ theme, toggleTheme }), [theme, toggleTheme])

    return (
        <ThemeContext.Provider value={value}>{children}</ThemeContext.Provider>
    )
}

// Define a type for a new user input
type NewUser = {
    name: string
    email: string
}

// Initial users data
const initialUsers: User[] = [
    { id: 1, name: 'Alice', email: 'alice@example.com' },
    { id: 2, name: 'Bob', email: 'bob@example.com' }
]

// UserList component that lists users
const UserList: React.FC<{ users: User[] }> = ({ users }) => {
    const { theme } = useTheme()
    return (
        <div>
            <h2 style={{ color: theme === 'dark' ? 'white' : 'black' }}>
                User List
            </h2>
            <ul>
                {users.map((user) => (
                    <li key={user.id}>
                        {user.name} ({user.email})
                    </li>
                ))}
            </ul>
        </div>
    )
}

// UserForm component to add a new user
const UserForm: React.FC<{ onAddUser: (newUser: NewUser) => void }> = ({
    onAddUser
}) => {
    const [name, setName] = useState('')
    const [email, setEmail] = useState('')

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault()
        if (name && email) {
            onAddUser({ name, email })
            setName('')
            setEmail('')
        }
    }

    return (
        <form onSubmit={handleSubmit}>
            <div>
                <label htmlFor="name">Name: </label>
                <input
                    id="name"
                    type="text"
                    value={name}
                    onChange={(e) => setName(e.target.value)}
                />
            </div>
            <div>
                <label htmlFor="email">Email: </label>
                <input
                    id="email"
                    type="email"
                    value={email}
                    onChange={(e) => setEmail(e.target.value)}
                />
            </div>
            <button type="submit">Add User</button>
        </form>
    )
}

// Main App component
const App: React.FC = () => {
    const [users, setUsers] = useState<User[]>(initialUsers)

    const addUser = useCallback((newUser: NewUser) => {
        setUsers((prevUsers) => [
            ...prevUsers,
            {
                id: prevUsers.length + 1,
                name: newUser.name,
                email: newUser.email
            }
        ])
    }, [])

    const removeUser = useCallback((id: number) => {
        setUsers((prevUsers) => prevUsers.filter((user) => user.id !== id))
    }, [])

    return (
        <ThemeProvider>
            <div>
                <h1>React TypeScript Example</h1>
                <UserForm onAddUser={addUser} />
                <UserList users={users} />
                <button onClick={() => removeUser(users[0].id)}>
                    Remove First User
                </button>
            </div>
        </ThemeProvider>
    )
}

// UserCard component to display individual user information
const UserCard: React.FC<User> = ({ id, name, email }) => {
    const { theme } = useTheme()

    return (
        <div
            style={{
                border: '1px solid',
                padding: '10px',
                margin: '10px 0',
                backgroundColor: theme === 'dark' ? '#333' : '#fff'
            }}
        >
            <p>ID: {id}</p>
            <p>Name: {name}</p>
            <p>Email: {email}</p>
        </div>
    )
}

// Utility function to fetch users (mocked)
const fetchUsers = async (): Promise<User[]> => {
    return new Promise((resolve) => {
        setTimeout(() => {
            resolve([
                { id: 1, name: 'Alice', email: 'alice@example.com' },
                { id: 2, name: 'Bob', email: 'bob@example.com' }
            ])
        }, 1000)
    })
}

// DataFetch component to demonstrate fetching data
const DataFetch: React.FC = () => {
    const [loading, setLoading] = useState(false)
    const [users, setUsers] = useState<User[]>([])

    useEffect(() => {
        setLoading(true)
        fetchUsers().then((data) => {
            setUsers(data)
            setLoading(false)
        })
    }, [])

    if (loading) return <div>Loading...</div>

    return (
        <div>
            <h2>Fetched Users</h2>
            {users.map((user) => (
                <UserCard
                    key={user.id}
                    id={user.id}
                    name={user.name}
                    email={user.email}
                />
            ))}
        </div>
    )
}

// Main application rendering
const RootApp: React.FC = () => {
    return (
        <div>
            <App />
            <DataFetch />
        </div>
    )
}

export default RootApp
