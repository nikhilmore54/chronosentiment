import { motion } from "framer-motion";

const fadeUp = {
  hidden: { opacity: 0, y: 40 },
  show: { opacity: 1, y: 0 }
};

export default function App() {

  const scriptURL = "https://script.google.com/macros/s/AKfycbzxN_eKDo3u-PFYrYkUXDOYCmXdRjpiw_fVP3MpK5_GRuECTyATMmoX0RG55bE_Fi5-/exec";

  const handleSubmit = async (e) => {
    e.preventDefault();

    const btn = e.target.querySelector("button");
    const status = document.getElementById("status");

    btn.innerText = "Submitting...";
    btn.disabled = true;

    try {
      await fetch(scriptURL, {
        method: "POST",
        headers: { "Content-Type": "text/plain" },
        body: JSON.stringify({
          name: e.target.name.value,
          email: e.target.email.value,
          company: e.target.company.value
        })
      });

      status.innerText = "✅ Demo request submitted!";
      e.target.reset();
    } catch {
      status.innerText = "❌ Something went wrong.";
    }

    btn.innerText = "Book My Demo";
    btn.disabled = false;
  };

  return (
    <div className="text-white bg-black min-h-screen font-sans selection:bg-indigo-500/30">
      {/* NAVBAR */}
      <nav className="fixed top-0 left-0 right-0 z-50 bg-black/50 backdrop-blur-md border-b border-white/5">
        <div className="max-w-6xl mx-auto p-4 flex justify-between items-center">
          <div className="flex items-center gap-2">
            <div className="w-8 h-8 bg-indigo-600 rounded-lg flex items-center justify-center font-bold text-white">C</div>
            <h1 className="text-xl font-bold tracking-tight">ClientSync</h1>
          </div>
          <div className="hidden md:flex gap-8 text-sm font-medium text-gray-400">
            <a href="#features" className="hover:text-white transition-colors">Features</a>
            <a href="#how-it-works" className="hover:text-white transition-colors">How it works</a>
            <a href="#pricing" className="hover:text-white transition-colors">Pricing</a>
          </div>
          <a href="#demo" className="bg-indigo-600 hover:bg-indigo-700 text-white px-5 py-2.5 rounded-full text-sm font-semibold transition-all hover:shadow-[0_0_20px_rgba(79,70,229,0.4)] active:scale-95">
            Book Demo
          </a>
        </div>
      </nav>

      {/* HERO */}
      <section className="relative pt-40 pb-24 px-6 overflow-hidden">
        {/* Abstract background glow */}
        <div className="absolute top-0 left-1/2 -translate-x-1/2 w-full max-w-4xl h-full bg-indigo-600/10 blur-[120px] rounded-full pointer-events-none" />
        
        <div className="relative max-w-4xl mx-auto text-center">
          <motion.div
            initial={{ opacity: 0, scale: 0.9 }}
            animate={{ opacity: 1, scale: 1 }}
            className="inline-block px-4 py-1.5 mb-6 rounded-full bg-indigo-500/10 border border-indigo-500/20 text-indigo-400 text-xs font-bold uppercase tracking-widest"
          >
            Empowering Modern Agencies
          </motion.div>
          
          <motion.h1 
            initial="hidden" 
            animate="show" 
            variants={fadeUp}
            className="text-6xl md:text-7xl font-bold leading-[1.1] tracking-tight text-white mb-8"
          >
            Stop Client Chaos.<br/>
            <span className="text-transparent bg-clip-text bg-gradient-to-r from-indigo-400 to-violet-400">
              Run Your Agency Like a System.
            </span>
          </motion.h1>

          <motion.p 
            initial={{ opacity: 0 }} 
            animate={{ opacity: 1 }} 
            transition={{ delay: 0.2 }}
            className="mt-6 text-gray-400 text-xl max-w-2xl mx-auto leading-relaxed"
          >
            Replace scattered WhatsApp, emails, and spreadsheets with one unified platform 
            designed for high-ticket client delivery.
          </motion.p>
          
          <motion.div 
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.3 }}
            className="mt-12 flex flex-col sm:flex-row gap-4 justify-center items-center"
          >
            <a href="#demo" className="w-full sm:w-auto px-8 py-4 bg-white text-black font-bold rounded-2xl hover:bg-gray-200 transition-all text-lg shadow-xl">
              Get Started for Free
            </a>
            <a href="#features" className="w-full sm:w-auto px-8 py-4 bg-white/5 border border-white/10 hover:bg-white/10 rounded-2xl transition-all font-semibold text-lg">
              View Features
            </a>
          </motion.div>
        </div>
      </section>

      {/* PROBLEM */}
      <section className="max-w-6xl mx-auto px-6 py-24">
        <div className="text-center mb-16">
          <h2 className="text-3xl md:text-4xl font-bold mb-4">Sound familiar?</h2>
          <p className="text-gray-500 max-w-lg mx-auto">Scaling an agency is hard when your operations are held together by sticky notes and prayers.</p>
        </div>

        <div className="grid md:grid-cols-3 gap-8 mt-12">
          {[
            { title: "Endless WhatsApp threads", desc: "Critical decisions buried under 500 unread messages in 20 different groups." },
            { title: "No task visibility", desc: "Zero clarity on who is doing what, when it's due, or if the client is happy." },
            { title: "Constant follow-ups", desc: "Wasting hours answering \"what's the update?\" instead of doing the actual work." }
          ].map((item, i) => (
            <motion.div 
              key={i} 
              variants={fadeUp} 
              initial="hidden" 
              whileInView="show"
              viewport={{ once: true }}
              className="group p-8 bg-gradient-to-b from-white/10 to-white/5 border border-white/10 rounded-3xl hover:border-indigo-500/50 transition-all duration-300"
            >
              <div className="w-12 h-12 bg-red-500/10 text-red-500 rounded-xl flex items-center justify-center mb-6 group-hover:scale-110 transition-transform">
                <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" /></svg>
              </div>
              <h3 className="text-xl font-bold mb-3">{item.title}</h3>
              <p className="text-gray-400 leading-relaxed">{item.desc}</p>
            </motion.div>
          ))}
        </div>
      </section>

      {/* FEATURES */}
      <section id="features" className="max-w-6xl mx-auto px-6 py-32">
        <div className="flex flex-col md:flex-row justify-between items-end gap-6 mb-16">
          <div className="max-w-xl">
            <h2 className="text-4xl font-bold mb-4">Everything in one place</h2>
            <p className="text-gray-500 text-lg">Professional-grade tools to manage every stage of the client lifecycle with surgical precision.</p>
          </div>
          <div className="hidden md:block h-[1px] flex-1 bg-white/10 mb-5 ml-8" />
        </div>

        <div className="grid md:grid-cols-2 gap-8">
          {[
            { title: "Client Dashboard", desc: "A white-labeled portal where clients see their project status and assets in one click." },
            { title: "Task Tracking", desc: "Kanban boards and timelines that keep your internal team synced and external clients informed." },
            { title: "Centralized Communication", desc: "Stop hunting through emails. All project communication is threaded and searchable." },
            { title: "Approvals Made Easy", desc: "One-click approvals for designs, copies, or strategies. No more missed feedback." }
          ].map((item, i) => (
            <motion.div 
              key={i} 
              variants={fadeUp} 
              initial="hidden" 
              whileInView="show"
              viewport={{ once: true }}
              className="p-10 bg-white/5 border border-white/10 rounded-[2.5rem] hover:bg-white/[0.08] transition-all cursor-default"
            >
              <h3 className="text-2xl font-bold text-indigo-400 mb-4">{item.title}</h3>
              <p className="text-gray-400 text-lg leading-relaxed">{item.desc}</p>
            </motion.div>
          ))}
        </div>
      </section>

      {/* HOW IT WORKS */}
      <section id="how-it-works" className="bg-zinc-950 py-32 border-y border-white/5">
        <div className="max-w-5xl mx-auto px-6">
          <div className="text-center mb-20">
            <h2 className="text-4xl font-bold">The Path to Efficiency</h2>
            <p className="text-gray-500 mt-4 text-lg">Go from chaos to systems in three simple steps.</p>
          </div>

          <div className="grid md:grid-cols-3 gap-16 relative">
            {/* Connector Line (Desktop) */}
            <div className="hidden md:block absolute top-12 left-0 w-full h-[1px] bg-indigo-500/20 z-0" />
            
            {[
              { step: "1", title: "Add Clients", desc: "Bring all your clients into one system with custom white-labeled access." },
              { step: "2", title: "Assign Work", desc: "Create tasks, assign team members, and set internal deadlines effortlessly." },
              { step: "3", title: "Track & Deliver", desc: "Monitor progress in real-time and deliver projects through the portal." }
            ].map((item, i) => (
              <div key={i} className="relative z-10">
                <div className="w-16 h-16 bg-indigo-600 text-white rounded-2xl flex items-center justify-center mb-8 text-2xl font-black shadow-[0_0_30px_rgba(79,70,229,0.3)]">
                  {item.step}
                </div>
                <h3 className="font-bold text-2xl mb-4 text-white">{item.title}</h3>
                <p className="text-gray-400 text-lg leading-relaxed">{item.desc}</p>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* SOCIAL PROOF */}
      <section className="text-center py-40 px-6 bg-gradient-to-b from-black via-indigo-900/10 to-black">
        <motion.div 
          initial={{ opacity: 0, scale: 0.95 }}
          whileInView={{ opacity: 1, scale: 1 }}
          className="max-w-4xl mx-auto"
        >
          <div className="flex justify-center gap-1 mb-8">
            {[1,2,3,4,5].map(s => <svg key={s} className="w-6 h-6 text-yellow-500 fill-current" viewBox="0 0 20 20"><path d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z" /></svg>)}
          </div>
          <p className="text-3xl md:text-4xl italic font-medium text-gray-200 leading-tight">
            “We reduced client follow-ups by 60% and improved delivery speed within 2 weeks.”
          </p>
          <div className="mt-10 flex items-center justify-center gap-4">
            <div className="w-12 h-12 bg-indigo-500/20 rounded-full border border-indigo-500/30 overflow-hidden" />
            <div className="text-left">
              <p className="text-white font-bold text-lg">Rahul S.</p>
              <p className="text-indigo-400 text-sm font-semibold uppercase tracking-wider">Founder, ScaleFlow Agency</p>
            </div>
          </div>
        </motion.div>
      </section>

      {/* PRICING */}
      <section id="pricing" className="text-center py-24 px-6">
        <h2 className="text-4xl font-bold mb-4">Simple, transparent pricing</h2>
        <p className="text-gray-500 mb-16 text-lg">Everything you need to run your agency professionally.</p>

        <motion.div 
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          className="inline-block w-full max-w-sm p-12 bg-white/5 border-2 border-indigo-600 rounded-[3rem] relative shadow-[0_0_50px_rgba(79,70,229,0.2)]"
        >
          <div className="absolute -top-5 left-1/2 -translate-x-1/2 bg-indigo-600 px-4 py-1.5 rounded-full text-xs font-black uppercase tracking-widest text-white shadow-lg">
            Agency Founder Special
          </div>
          <h3 className="text-2xl font-bold text-gray-300 mb-2">Growth Plan</h3>
          <div className="flex items-baseline justify-center gap-1 mt-6">
            <span className="text-6xl font-black text-white">₹999</span>
            <span className="text-xl text-gray-500 font-medium">/mo</span>
          </div>
          <div className="mt-10 space-y-5 text-left border-t border-white/10 pt-10">
            {[
              "Up to 10 Active Clients",
              "Unlimited Internal Projects",
              "Team Collaboration Tools",
              "10GB Cloud Storage",
              "Priority Email Support"
            ].map(f => (
              <div key={f} className="flex items-center gap-3 text-gray-300">
                <svg className="w-5 h-5 text-indigo-500" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={3} d="M5 13l4 4L19 7" /></svg>
                <span className="font-medium">{f}</span>
              </div>
            ))}
          </div>
          <a href="#demo" className="mt-12 block w-full bg-indigo-600 py-5 rounded-2xl font-black text-xl hover:bg-indigo-700 transition-all shadow-lg hover:shadow-indigo-500/20 active:scale-[0.98]">
            Get Started Now
          </a>
        </motion.div>
      </section>

      {/* DEMO */}
      <section id="demo" className="relative text-center py-40 px-6 bg-zinc-950 overflow-hidden">
        {/* Background effect */}
        <div className="absolute bottom-0 left-0 right-0 h-1/2 bg-gradient-to-t from-indigo-900/20 to-transparent pointer-events-none" />
        
        <div className="relative max-w-xl mx-auto">
          <h2 className="text-5xl font-black mb-6">Ready to scale?</h2>
          <p className="text-gray-400 text-xl mb-12 leading-relaxed">Book a 15-minute walkthrough to see how ClientSync can save you 20+ hours a week.</p>

          <form onSubmit={handleSubmit} className="space-y-4">
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
              <input name="name" placeholder="Full Name" required className="w-full p-5 rounded-2xl bg-black border border-white/10 focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500 outline-none transition-all placeholder:text-gray-600"/>
              <input name="email" type="email" placeholder="Email Address" required className="w-full p-5 rounded-2xl bg-black border border-white/10 focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500 outline-none transition-all placeholder:text-gray-600"/>
            </div>
            <input name="company" placeholder="Agency Name" className="w-full p-5 rounded-2xl bg-black border border-white/10 focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500 outline-none transition-all placeholder:text-gray-600"/>

            <button type="submit" className="group w-full bg-white text-black py-5 rounded-2xl font-black text-xl hover:bg-gray-100 transition-all shadow-2xl flex items-center justify-center gap-2">
              Book My Free Demo
              <svg className="w-6 h-6 group-hover:translate-x-1 transition-transform" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={3} d="M17 8l4 4m0 0l-4 4m4-4H3" /></svg>
            </button>

            <p id="status" className="mt-4 font-bold text-indigo-400 h-6"></p>
          </form>
          
          <p className="mt-8 text-gray-500 text-sm">Join 50+ agencies running more profitably on ClientSync.</p>
        </div>
      </section>

      {/* FOOTER */}
      <footer className="py-20 px-6 border-t border-white/5 bg-black">
        <div className="max-w-6xl mx-auto flex flex-col md:flex-row justify-between items-center gap-8">
          <div className="flex items-center gap-2 opacity-60">
            <div className="w-6 h-6 bg-indigo-600 rounded flex items-center justify-center font-bold text-[10px]">C</div>
            <p className="font-bold">ClientSync</p>
          </div>
          <div className="flex gap-8 text-gray-500 text-sm">
            <a href="#" className="hover:text-white transition-colors">Privacy Policy</a>
            <a href="#" className="hover:text-white transition-colors">Terms of Service</a>
            <a href="#" className="hover:text-white transition-colors">Twitter / X</a>
          </div>
          <p className="text-gray-600 text-sm">© 2026 ClientSync. Crafted for Agency Founders.</p>
        </div>
      </footer>
    </div>
  );
}
