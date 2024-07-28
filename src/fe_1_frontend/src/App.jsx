import { useState, useEffect } from 'react';
import { fe_1_backend } from 'declarations/fe_1_backend';

function App() {
  const [greeting, setGreeting] = useState('');
  const [proposals, setProposals] = useState([]);
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");

  function handleSubmit(event) {
    event.preventDefault();
    const name = event.target.elements.name.value;
    fe_1_backend.greet(name).then((greeting) => {
      setGreeting(greeting);
    });
    return false;
  }
  useEffect(() => {
    const fetchProposals = async () => {
      try {
        const allProposals = await fe_1_backend.get_all_proposals();
        setProposals(allProposals);
      }catch(e){
        console.log(e);
      }

    }
 

    fetchProposals();
  }, [proposals]);

  const addProposal = async (e) => {
    e.preventDefault();
    const title = e.target[0].value;
    const description = e.target[1].value;

    // Prepare payload
    const proposalPayload = {
      title,
      description,
    };

    await fe_1_backend.add_proposal(proposalPayload);

    // Refresh proposals list
    const allProposals = await fe_1_backend.get_all_proposals();
    setProposals(allProposals);

    // Clear input fields
    setTitle("");
    setDescription("");
  };
  return (
    <main>

      <form onSubmit={addProposal}>
        <label>
          Title:
          <input type="text" value={title} onChange={(e) => setTitle(e.target.value)} required />
        </label>
        <br />
        <label>
          Description:
          <input type="text" value={description} onChange={(e) => setDescription(e.target.value)} required />
        </label>
        <br />

        <br />
        <button type="submit">Add Proposal</button>
      </form>
      <section>
        <h2>Proposals</h2>
        {proposals.length > 0 ? (
          <ul>
            {proposals.map((proposal, index) => (
              <li key={index}>
                <h3>{proposal.title}</h3>
                <p>{proposal.description}</p>
                <ul>
                  
                </ul>
              </li>
            ))}
          </ul>
        ) : (
          <p>No proposals found.</p>
        )}
      </section>
    </main>
  );
}

export default App;
