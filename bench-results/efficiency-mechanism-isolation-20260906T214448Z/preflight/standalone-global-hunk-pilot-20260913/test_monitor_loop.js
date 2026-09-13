const AF=Object.getPrototypeOf(async function(){}).constructor;
const run=new AF("tools","load","store","notify","text","exit","setTimeout",load("global_monitor_code"));
const rows=[];
for(const mode of ["normal","threshold","sampler-error","long-gap","launch-receipt-failure","first-control-failure"]){
  let publications=0,stops=0,sampled=0,state={};
  const fake={
    exec_command:async({cmd})=>{
      if(cmd==="launch")return{session_id:9};
      if(cmd.startsWith("rg"))return{exit_code:0,output:"**Live resource monitor:** mock"};
      sampled++;
      if(mode==="sampler-error")return{exit_code:1,output:"synthetic error"};
      return{exit_code:0,output:JSON.stringify({
        at_utc:new Date(100000+(sampled-1)*(mode==="long-gap"?70000:30000)).toISOString(),
        runs:[{reported_raw:25000000,completed_public_turns:sampled}],
        stop_wave:mode==="threshold"||mode==="first-control-failure",
        phase_terminal:sampled>=(mode==="normal"?2:3)})};
    },
    apply_patch:async()=>{
      publications++;
      if(mode==="launch-receipt-failure"&&publications===1)throw new Error("receipt unavailable");
    },
    write_stdin:async({chars})=>{
      if(chars==="\u0003"){
        stops++;
        if(mode==="first-control-failure"&&stops===1)throw new Error("transient control error");
      }
      return{exit_code:0};
    }
  };
  let error=null;
  try{
    await run(fake,k=>({global_P:"phase",global_resource_cmd:"sample",global_launch_command:"launch"}[k]),
      (k,v)=>state[k]=v,()=>{},()=>{},()=>{throw new Error("EXPECTED_EXIT")},cb=>{cb();return 0;});
  }catch(e){if(e.message!=="EXPECTED_EXIT")throw e;error=e.message;}
  const expected=mode==="normal"?0:mode==="first-control-failure"?2:1;
  if(stops!==expected)throw new Error(mode+": "+stops+" control attempts, expected "+expected);
  rows.push({mode,samples:sampled,control_attempts:stops,expected,pass:true});
}
store("global_monitor_green",rows);text(rows);
