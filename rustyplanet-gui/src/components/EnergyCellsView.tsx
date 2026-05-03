import { Box, Stack, Typography } from '@mui/material'

interface EnergyCellsViewProps {
  total: number
  charged: number
  resourceTotal?: number
  resourceCharged?: number
  defenseTotal?: number
  defenseCharged?: number
}

export function EnergyCellsView({
  total,
  charged,
  resourceTotal,
  resourceCharged,
  defenseTotal,
  defenseCharged,
}: EnergyCellsViewProps) {
  const safeResourceTotal = resourceTotal ?? total
  const safeResourceCharged = resourceCharged ?? charged
  const safeDefenseTotal = defenseTotal ?? 0
  const safeDefenseCharged = defenseCharged ?? 0

  const renderRow = (count: number, chargedCount: number, color: string) => {
    return (
      <Stack direction="row" spacing={0.5}>
        {Array.from({ length: count }).map((_, idx) => {
          const isCharged = idx < chargedCount
          return (
            <Box
              key={idx}
              sx={{
                width: 18,
                height: 10,
                borderRadius: 0.5,
                border: '1px solid',
                borderColor: isCharged ? color : 'grey.700',
                bgcolor: isCharged ? color : 'transparent',
              }}
            />
          )
        })}
      </Stack>
    )
  }

  return (
    <Stack spacing={1}>
      <Typography variant="body2" color="text.secondary">
        Total charged: {charged}/{total}
      </Typography>
      <Stack spacing={0.5}>
        <Typography variant="caption" color="text.secondary">
          Resource cells
        </Typography>
        {renderRow(safeResourceTotal, safeResourceCharged, '#81c784')}
      </Stack>
      {safeDefenseTotal > 0 && (
        <Stack spacing={0.5}>
          <Typography variant="caption" color="text.secondary">
            Defense cells
          </Typography>
          {renderRow(safeDefenseTotal, safeDefenseCharged, '#64b5f6')}
        </Stack>
      )}
    </Stack>
  )
}

