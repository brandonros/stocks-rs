import fs from 'fs'
import { globSync } from 'glob'

const run = async () => {
  const filenames = globSync('./output/backtest-result-*.json')
  console.log('date,profit_limit_percentage,stop_loss_percentage,slippage_percentage,fast_periods,slow_periods,fast_slow_pair,simple_profit_loss_percentage,compounded_profit_loss_percentage,num_trades')
  for (const filename of filenames) {
    const input = fs.readFileSync(`./${filename}`).toString('utf8')
    const parsedInput = JSON.parse(input)
    const date = filename.replace('.json', '')
    const best_combination_result = parsedInput[0].combination_result // sorted z-a on highest profit first?
    const profit_limit_percentage = best_combination_result.backtest_context.profit_limit_percentage
    const stop_loss_percentage = best_combination_result.backtest_context.stop_loss_percentage
    const slippage_percentage = best_combination_result.backtest_context.slippage_percentage
    const fast_periods = best_combination_result.trade_generation_context.fast_periods
    const slow_periods = best_combination_result.trade_generation_context.slow_periods
    const fast_slow_pair = `${fast_periods}:${slow_periods}`
    const simple_profit_loss_percentage = best_combination_result.simple_profit_loss_percentage
    const compounded_profit_loss_percentage = best_combination_result.compounded_profit_loss_percentage
    const num_trades = best_combination_result.num_trades
    console.log(`${date},${profit_limit_percentage},${stop_loss_percentage},${slippage_percentage},${fast_periods},${slow_periods},${fast_slow_pair},${simple_profit_loss_percentage},${compounded_profit_loss_percentage},${num_trades}`)
  }
}

run()
